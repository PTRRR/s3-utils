use clap::Parser;
use futures_util::StreamExt;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub f2b: Option<String>,

    #[arg(long)]
    pub local_path: String,

    #[arg(long)]
    pub target_bucket: String,

    #[arg(long)]
    pub target_region: String,

    #[arg(long)]
    pub target_aws_access_key_id: String,

    #[arg(long)]
    pub target_aws_secret_access_key: String,

    #[arg(long)]
    pub target_endpoint: Option<String>,

    #[arg(long)]
    pub concurrency: Option<usize>,
}

pub async fn folder_to_bucket() -> Result<(), Box<dyn std::error::Error>> {
    // Read command-line arguments
    let args = Args::parse();
    let concurrency = args.concurrency.unwrap_or(50);
    let target_bucket = args.target_bucket;
    let target_endpoint = args
        .target_endpoint
        .unwrap_or(format!("s3.{}.amazonaws.com", args.target_region.clone()));

    // Create AWS credentials provider
    let target_credentials_provider = StaticProvider::new_minimal(
        args.target_aws_access_key_id,
        args.target_aws_secret_access_key,
    );

    // Create S3 clients for origin and target regions
    let target_client = S3Client::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        target_credentials_provider,
        Region::Custom {
            name: args.target_region.clone(),
            endpoint: target_endpoint,
        },
    );

    let (tx, rx) = mpsc::channel(concurrency);

    tokio::spawn(async move {
        for file in WalkDir::new(args.local_path).into_iter() {
            let file = file.unwrap();
            if file.file_type().is_file() {
                tx.send(file).await.unwrap();
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
        .for_each_concurrent(concurrency, |dir_entry| {
            let target_client = target_client.clone();
            let target_bucket = target_bucket.clone();

            async move {
                let file_path = format!("{}", dir_entry.path().display());
                let object_key = dir_entry.file_name().to_str().unwrap();

                // Upload the file to the target bucket
                let mut file = File::open(file_path.clone()).await.unwrap();
                let mut vec = Vec::new();
                let _ = file.read_to_end(&mut vec).await;

                let put_request = PutObjectRequest {
                    bucket: target_bucket.to_owned(),
                    key: object_key.into(),
                    body: Some(vec.into()),
                    ..Default::default()
                };

                match target_client.put_object(put_request).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error uploading file {}: {:?}", file_path, e);
                    }
                }

                println!("File sync: {}", file_path);
            }
        })
        .await;

    return Ok(());
}
