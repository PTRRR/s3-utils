use futures_util::{lock::Mutex, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

use rusoto_s3::{
    CompleteMultipartUploadRequest, CompletedMultipartUpload, CompletedPart,
    CreateMultipartUploadRequest, PutObjectRequest, S3Client, UploadPartRequest, S3,
};
use tokio::{fs::File, io::AsyncReadExt};
use walkdir::DirEntry;

const BUFFER_SIZE: usize = 2 * 1024 * 1024;
const PART_SIZE: usize = 20 * 1024 * 1024;

trait ValueOrReferenceTo<T> {
    fn as_ref(&self) -> &T;
}

impl<'a, T> ValueOrReferenceTo<T> for &'a T {
    fn as_ref(&self) -> &T {
        *self
    }
}

impl<T> ValueOrReferenceTo<T> for T {
    fn as_ref(&self) -> &T {
        self
    }
}

pub async fn get_file_size(file: &File) -> u64 {
    let metadata = file.metadata().await.unwrap();
    return metadata.len();
}

pub fn get_file_key<S>(file: &DirEntry, root_directory: Option<S>) -> String
where
    S: Into<String>,
{
    let file_path = file.path().to_str().unwrap().to_owned();
    let key = match root_directory {
        Some(root_directory) => file_path
            .replace(&root_directory.into(), "")
            .replace("\\", "/"),
        _ => file.file_name().to_str().unwrap().to_owned(),
    };

    let key = match key.chars().next() {
        Some('/') => key[1..].to_owned(),
        _ => key,
    };

    return key;
}

#[derive(Debug)]
struct UploadPart {
    part_number: usize,
    body: Vec<u8>,
}

pub async fn upload_s3_object<K, B>(
    file: File,
    key: K,
    bucket: B,
    client: &S3Client,
) -> Result<(), Box<dyn std::error::Error>>
where
    K: Into<String> + Clone,
    B: Into<String> + Clone,
{
    let size = get_file_size(&file).await;
    let is_multipart = size > PART_SIZE.try_into().unwrap();

    if is_multipart {
        let create_multipart_request = CreateMultipartUploadRequest {
            bucket: bucket.clone().into(),
            key: key.clone().into(),
            ..Default::default()
        };
        let create_multipart_response = client
            .create_multipart_upload(create_multipart_request)
            .await
            .unwrap();
        let upload_id = create_multipart_response.upload_id.unwrap();

        let parts = Arc::new(Mutex::new(Vec::new()));
        let capacity = (size as f64 / BUFFER_SIZE as f64).ceil() as usize;
        let (tx, rx) = mpsc::channel(capacity);

        tokio::spawn(async move {
            let mut file = file;
            let mut part_number = 1;
            let mut buffer = vec![0u8; BUFFER_SIZE];
            let mut bytes_read = file.read(&mut buffer).await.unwrap();
            let mut body = Vec::new();
            while bytes_read > 0 {
                body.append(&mut buffer[..bytes_read].to_vec());
                if body.len() >= PART_SIZE || bytes_read < BUFFER_SIZE {
                    tx.send(UploadPart {
                        part_number,
                        body: body.clone(),
                    })
                    .await
                    .unwrap();
                    part_number += 1;
                    body.clear();
                }
                bytes_read = file.read(&mut buffer).await.unwrap();
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
            .for_each_concurrent(100, |upload_part| {
                let bucket = bucket.clone();
                let key = key.clone();
                let upload_id = upload_id.clone();
                let client = client.clone();
                let parts = parts.clone();

                async move {
                    let upload_part_request = UploadPartRequest {
                        bucket: bucket.clone().into(),
                        key: key.clone().into(),
                        part_number: upload_part.part_number as i64,
                        upload_id: upload_id.to_string(),
                        body: Some(upload_part.body.into()),
                        ..Default::default()
                    };

                    let upload_part_response =
                        client.upload_part(upload_part_request).await.unwrap();
                    println!("Uploading part: {}", upload_part.part_number);
                    let etag = upload_part_response.e_tag.unwrap();
                    parts.lock().await.push(CompletedPart {
                        e_tag: Some(etag),
                        part_number: Some(upload_part.part_number as i64),
                    });
                }
            })
            .await;

        let sorted_parts = parts.lock().await.as_ref().to_vec();
        let mut sorted_parts = sorted_parts.clone();
        sorted_parts.sort_by(|a, b| a.part_number.cmp(&b.part_number));

        let complete_multipart_request = CompleteMultipartUploadRequest {
            bucket: bucket.clone().into(),
            key: key.clone().into(),
            upload_id: upload_id.to_string(),
            multipart_upload: Some(CompletedMultipartUpload {
                parts: Some(sorted_parts),
            }),
            ..Default::default()
        };

        let _complete_multipart_response = client
            .complete_multipart_upload(complete_multipart_request)
            .await
            .unwrap();
    } else {
        let mut file = file;
        let mut buffer = vec![0u8; size as usize];
        let _bytes_read = file.read(&mut buffer).await.unwrap();

        let put_request = PutObjectRequest {
            bucket: bucket.into(),
            key: key.into(),
            body: Some(buffer.into()),
            ..Default::default()
        };

        let _put_response = client.put_object(put_request).await.unwrap();
    }

    return Ok(());
}
