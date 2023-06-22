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
| `--origin-bucket`                | The name of the source bucket.                                |
| `--target-bucket`                | The name of the target bucket.                                |
| `--origin-region`                | The AWS region of the source bucket.                          |
| `--target-region`                | The AWS region of the target bucket.                          |
| `--origin-aws-access-key-id`     | The AWS access key ID for the source bucket.                  |
| `--target-aws-access-key-id`     | The AWS access key ID for the target bucket.                  |
| `--origin-aws-secret-access-key` | The AWS secret access key for the source bucket.              |
| `--target-aws-secret-access-key` | The AWS secret access key for the target bucket.              |
| `--origin-endpoint`              | (Optional) Custom endpoint URL for the source bucket.         |
| `--target-endpoint`              | (Optional) Custom endpoint URL for the target bucket.         |
| `--concurrency`                  | (Optional) Number of concurrent file transfers (default: 50). |

### Example

```sh
cargo run -- b2b --origin-bucket <source-bucket> --target-bucket <target-bucket> --origin-region <source-region> --target-region <target-region> --origin-aws-access-key-id <source-access-key-id> --target-aws-access-key-id <target-access-key-id> --origin-aws-secret-access-key <source-secret-access-key> --target-aws-secret-access-key <target-secret-access-key [--origin-endpoint <source-custom-endpoint>] [--target-endpoint <target-custom-endpoint>] [--concurrency <num-concurrent-transfers>]

```
