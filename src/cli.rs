use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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
