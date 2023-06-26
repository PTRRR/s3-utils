mod b2b;
mod cli;
mod delete_bucket;
mod f2b;

use b2b::bucket_to_bucket;
use delete_bucket::delete_bucket;
use f2b::folder_to_bucket;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let method = args.get(1);

    if method.is_none() {
        println!("No method specified");
        return;
    }

    let method = method.unwrap();

    match method.as_str() {
        "b2b" => {
            let _ = bucket_to_bucket().await;
        }
        "f2b" => {
            let _ = folder_to_bucket().await;
        }
        "delete_bucket" => {
            let _ = delete_bucket().await;
        }
        _ => {
            println!("Unknown method: {}", method);
        }
    }
}
