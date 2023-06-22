use clap::Parser;
use futures_util::StreamExt;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub b2b: Option<String>,

    #[arg(long)]
    pub origin_bucket: String,

    #[arg(long)]
    pub target_bucket: String,

    #[arg(long)]
    pub origin_region: String,

    #[arg(long)]
    pub target_region: String,

    #[arg(long)]
    pub origin_aws_access_key_id: String,

    #[arg(long)]
    pub target_aws_access_key_id: String,

    #[arg(long)]
    pub origin_aws_secret_access_key: String,

    #[arg(long)]
    pub target_aws_secret_access_key: String,

    #[arg(long)]
    pub origin_endpoint: Option<String>,

    #[arg(long)]
    pub target_endpoint: Option<String>,

    #[arg(long)]
    pub concurrency: Option<usize>,
}

pub async fn bucket_to_bucket() -> Result<(), Box<dyn std::error::Error>> {
    // Read command-line arguments
    let args = Args::parse();
    let concurrency = args.concurrency.unwrap_or(50);
    let origin_bucket = args.origin_bucket;
    let target_bucket = args.target_bucket;
    let origin_endpoint = args
        .origin_endpoint
        .unwrap_or(format!("s3.{}.amazonaws.com", args.origin_region.clone()));
    let target_endpoint = args
        .target_endpoint
        .unwrap_or(format!("s3.{}.amazonaws.com", args.target_region.clone()));

    // Create AWS credentials provider
    let origin_credentials_provider = StaticProvider::new_minimal(
        args.origin_aws_access_key_id,
        args.origin_aws_secret_access_key,
    );

    let target_credentials_provider = StaticProvider::new_minimal(
        args.target_aws_access_key_id,
        args.target_aws_secret_access_key,
    );

    // Create S3 clients for origin and target regions
    let origin_client = S3Client::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        origin_credentials_provider,
        Region::Custom {
            name: args.origin_region.clone(),
            endpoint: origin_endpoint,
        },
    );

    let target_client = S3Client::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        target_credentials_provider,
        Region::Custom {
            name: args.target_region.clone(),
            endpoint: target_endpoint,
        },
    );

    // List objects in the origin bucket
    let list_objects_result = origin_client
        .list_objects_v2(rusoto_s3::ListObjectsV2Request {
            bucket: origin_bucket.to_string(),
            ..Default::default()
        })
        .await;

    let (tx, rx) = mpsc::channel(concurrency);

    tokio::spawn(async move {
        match list_objects_result {
            Ok(output) => {
                if let Some(objects) = output.contents {
                    for object in objects {
                        tx.send(object.key.unwrap()).await.unwrap();
                    }
                }
            }
            Err(e) => {
                println!("Error listing objects: {:?}", e);
            }
        };
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
        .for_each_concurrent(concurrency, |object_key| {
            let object_key = object_key.clone();
            let origin_client = origin_client.clone();
            let origin_bucket = origin_bucket.clone();
            let target_client = target_client.clone();
            let target_bucket = target_bucket.clone();

            async move {
                let download_request = GetObjectRequest {
                    bucket: origin_bucket.to_owned(),
                    key: object_key.clone(),
                    ..Default::default()
                };

                // Send the request and download the file
                let object = origin_client.get_object(download_request).await.ok();

                if object.is_some() {
                    let file_path = format!("{}", object_key);
                    let mut file = File::create(file_path.clone()).await.unwrap();

                    // Read the object data and write it to the file
                    let stream = object.unwrap().body.unwrap();
                    tokio::io::copy(&mut stream.into_async_read(), &mut file)
                        .await
                        .expect("Failed to download file");

                    // Upload the file to the target bucket
                    let mut file = File::open(file_path.clone()).await.unwrap();
                    let mut vec = Vec::new();
                    let _ = file.read_to_end(&mut vec).await;

                    let put_request = PutObjectRequest {
                        bucket: target_bucket.to_owned(),
                        key: object_key.to_owned(),
                        body: Some(vec.into()),
                        ..Default::default()
                    };

                    match target_client.put_object(put_request).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error uploading file {}: {:?}", file_path, e);
                        }
                    }

                    match tokio::fs::remove_file(file_path.clone()).await {
                        Ok(_) => {
                            println!("File sync: {}", file_path);
                        }
                        Err(e) => {
                            println!("Error deleting file {}: {:?}", file_path, e);
                        }
                    }
                }
            }
        })
        .await;

    return Ok(());
}
