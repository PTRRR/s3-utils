use clap::Parser;
use futures_util::StreamExt;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{DeleteObjectRequest, S3Client, S3};
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
    let concurrency = args.concurrency.unwrap_or(50);
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

    // List objects in the origin bucket
    let mut list_objects_result = Vec::new();

    let mut objects = client
        .list_objects_v2(rusoto_s3::ListObjectsV2Request {
            bucket: bucket.to_string(),
            ..Default::default()
        })
        .await;

    let mut continuation_token: Option<String> = objects.unwrap().continuation_token;

    // println!(
    //     "Listing objects in bucket {:?}",
    //     objects.unwrap().next_continuation_token
    // );

    while continuation_token.is_some() {
        match objects.unwrap().contents {
            Some(contents) => {
                for object in contents {
                    println!("Deleting {:?}", object.key);
                    list_objects_result.push(object);
                }
            }
            _ => {}
        }

        objects = client
            .list_objects_v2(rusoto_s3::ListObjectsV2Request {
                bucket: bucket.to_string(),
                ..Default::default()
            })
            .await;
    }

    // while let Ok(output) = objects {
    //     match output.contents {
    //         Some(contents) => {
    //             for object in contents {
    //                 println!("Deleting {:?}", object.key);
    //                 list_objects_result.push(object);
    //             }
    //         }
    //         _ => {}
    //     }

    //     objects = client
    //         .list_objects_v2(rusoto_s3::ListObjectsV2Request {
    //             bucket: bucket.to_string(),
    //             ..Default::default()
    //         })
    //         .await;
    // }

    println!("Found {} objects", list_objects_result.len());

    let (tx, rx) = mpsc::channel(concurrency);

    tokio::spawn(async move {
        for object in list_objects_result {
            tx.send(object.key.unwrap()).await.unwrap();
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
        .for_each_concurrent(concurrency, |object_key| {
            let object_key = object_key.clone();
            let client = client.clone();
            let bucket = bucket.clone();

            async move {
                let delete_object_request = DeleteObjectRequest {
                    bucket: bucket.to_owned(),
                    key: object_key.clone(),
                    bypass_governance_retention: Some(true),
                    ..Default::default()
                };

                let res: Option<rusoto_s3::DeleteObjectOutput> =
                    client.delete_object(delete_object_request).await.ok();

                println!("Deleted {:?}", res);

                println!("Deleted {}", object_key);
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
