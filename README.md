# S3 Utils

## Prerequisites

Before running the script, make sure you have the following prerequisites:

- Rust programming language installed (https://www.rust-lang.org/tools/install)
- AWS account with access to the source and target S3 buckets
- AWS access key ID and secret access key for both buckets
- Familiarity with AWS regions and custom endpoints (if applicable)

## Usage

1. Clone the repository:

   ```shell
   git clone <repository_url>
   ```

## S3 Bucket-to-Bucket File Transfer

---

This Rust script allows you to transfer files from one Amazon S3 bucket to another. It utilizes the Rusoto library for interacting with AWS services and Tokio for asynchronous execution. The script provides a command-line interface (CLI) that accepts various options to configure the source and target buckets, AWS credentials, and other settings.

### The script accepts the following command-line arguments:

| Argument                         | Description                                                   |
| -------------------------------- | ------------------------------------------------------------- |
| `--origin_bucket`                | The name of the source bucket.                                |
| `--target_bucket`                | The name of the target bucket.                                |
| `--origin_region`                | The AWS region of the source bucket.                          |
| `--target_region`                | The AWS region of the target bucket.                          |
| `--origin_aws_access_key_id`     | The AWS access key ID for the source bucket.                  |
| `--target_aws_access_key_id`     | The AWS access key ID for the target bucket.                  |
| `--origin_aws_secret_access_key` | The AWS secret access key for the source bucket.              |
| `--target_aws_secret_access_key` | The AWS secret access key for the target bucket.              |
| `--origin_endpoint`              | (Optional) Custom endpoint URL for the source bucket.         |
| `--target_endpoint`              | (Optional) Custom endpoint URL for the target bucket.         |
| `--concurrency`                  | (Optional) Number of concurrent file transfers (default: 50). |

### Example

```sh
cargo run -- b2b --origin_bucket <source_bucket> --target_bucket <target_bucket> --origin_region <source_region> --target_region <target_region> --origin_aws_access_key_id <source_access_key_id> --target_aws_access_key_id <target_access_key_id> --origin_aws_secret_access_key <source_secret_access_key> --target_aws_secret_access_key <target_secret_access_key [--origin_endpoint <source_custom_endpoint>] [--target_endpoint <target_custom_endpoint>] [--concurrency <num_concurrent_transfers>]

```
