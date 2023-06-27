use clap::Parser;
use futures_util::StreamExt;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{DeleteObjectRequest, ListObjectsV2Request, S3Client, S3};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub db: Option<String>,

    #[arg(long)]
    pub bucket: String,

    #[arg(long)]
    pub region: String,

    #[arg(long)]
    pub aws_access_key_id: String,

    #[arg(long)]
    pub aws_secret_access_key: String,

    #[arg(long)]
    pub endpoint: Option<String>,

    #[arg(long)]
    pub concurrency: Option<usize>,
}

pub async fn delete_bucket() -> Result<(), Box<dyn std::error::Error>> {
    // Read command-line arguments
    let args = Args::parse();
    let concurrency = args.concurrency.unwrap_or(10);
    let bucket = args.bucket;
    let endpoint = args
        .endpoint
        .unwrap_or(format!("s3.{}.amazonaws.com", args.region.clone()));

    // Create AWS credentials provider
    let credentials_provider =
        StaticProvider::new_minimal(args.aws_access_key_id, args.aws_secret_access_key);

    // Create S3 clients for origin and target regions
    let client = S3Client::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        credentials_provider,
        Region::Custom {
            name: args.region.clone(),
            endpoint,
        },
    );

    let (tx, rx) = mpsc::channel(concurrency);
    tokio::spawn({
        let client = client.clone();
        let bucket = bucket.clone();

        async move {
            let mut continuation_token = None;
            loop {
                let list_objects_request = ListObjectsV2Request {
                    bucket: bucket.to_owned(),
                    continuation_token: continuation_token.clone(),
                    ..Default::default()
                };

                let list_objects_output =
                    client.list_objects_v2(list_objects_request).await.unwrap();

                if let Some(contents) = list_objects_output.contents {
                    for object in contents {
                        tx.send(object.key.unwrap()).await.unwrap();
                    }
                }

                if !list_objects_output.is_truncated.unwrap_or_default() {
                    break;
                }

                continuation_token = list_objects_output.next_continuation_token;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
        .for_each_concurrent(concurrency, |key| {
            let key = key.clone();
            let client = client.clone();
            let bucket = bucket.clone();

            async move {
                let delete_object_request = DeleteObjectRequest {
                    bucket: bucket.to_owned(),
                    key: key.clone(),
                    bypass_governance_retention: Some(true),
                    ..Default::default()
                };

                client.delete_object(delete_object_request).await.ok();
                println!("Deleted {}", key);
            }
        })
        .await;

    let delete_bucket_request = rusoto_s3::DeleteBucketRequest {
        bucket: bucket.to_owned(),
        ..Default::default()
    };

    let _ = client.delete_bucket(delete_bucket_request).await.ok();

    return Ok(());
}
