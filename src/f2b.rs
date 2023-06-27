use clap::Parser;
use futures_util::StreamExt;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub f2b: Option<String>,

    #[arg(long)]
    pub directory: String,

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

    #[arg(long)]
    pub flatten: Option<bool>,
}

pub async fn folder_to_bucket() -> Result<(), Box<dyn std::error::Error>> {
    // Read command-line arguments
    let args = Args::parse();
    let concurrency = args.concurrency.unwrap_or(50);
    let bucket = args.bucket;
    let endpoint = args
        .endpoint
        .unwrap_or(format!("s3.{}.amazonaws.com", args.region.clone()));
    let directory = args.directory.clone();

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
        let directory = directory.clone();
        async move {
            for file in WalkDir::new(directory).into_iter() {
                let file = file.unwrap();
                if file.file_type().is_file() {
                    tx.send(file).await.unwrap();
                }
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
        .for_each_concurrent(concurrency, |file| {
            let directory = directory.clone();
            let client = client.clone();
            let bucket = bucket.clone();
            let root_dir = PathBuf::from(directory);
            let root_dir = root_dir.canonicalize().unwrap();

            async move {
                let file_path = format!("{}", file.path().display());
                let key = match args.flatten {
                    Some(true) => file.file_name().to_str().unwrap().to_owned(),
                    _ => file_path
                        .replace(root_dir.to_str().unwrap(), "")
                        .replace("\\", "/"),
                };

                // Remove first character if it is a slash
                let key = match key.chars().next() {
                    Some('/') => key[1..].to_owned(),
                    _ => key,
                };

                // Upload the file to the target bucket
                let mut file = File::open(file_path.clone()).await.unwrap();
                let mut vec = Vec::new();
                let _ = file.read_to_end(&mut vec).await;

                let put_request = PutObjectRequest {
                    bucket: bucket.to_owned(),
                    key: key.clone().into(),
                    body: Some(vec.into()),
                    ..Default::default()
                };

                match client.put_object(put_request).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error uploading file {}: {:?}", file_path, e);
                    }
                }

                println!("File sync: {}", key);
            }
        })
        .await;

    return Ok(());
}
