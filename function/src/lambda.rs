use crate::{presign_get_object, upload_to_s3, visualize_audio};
use anyhow::Error;
use env_logger;
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use uuid::Uuid;
use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;

// how long each preset should be shown in the visualization
const DEFAULT_PRESET_DURATION: u32 = 10;

#[derive(Deserialize, Debug)]
pub struct Args {
    input_url: String,
    preset_duration: Option<u32>,
    resolution: String,
}

#[derive(Serialize)]
pub struct LambdaResponse {
    message: String,
    output_video_url: Option<String>,
}

pub async fn lambda_handler(event: LambdaEvent<LambdaFunctionUrlRequest>) -> Result<LambdaResponse, LambdaError> {
    // parse body as JSON
    let event: Args = serde_json::from_str(&event.payload.body.expect("No body in request"))
        .map_err(|e| LambdaError::from(e.to_string()))?;

    // log input args
    log::info!("Input args: {:?}", event);

    // validate the input
    if event.input_url.is_empty() {
        return Err(LambdaError::from("Input url is required"));
    }
    let preset_duration = event
        .preset_duration
        .or(Some(DEFAULT_PRESET_DURATION))
        .unwrap();

    // get OUTPUT_BUCKET from environment variable or panic
    let s3_bucket =
        env::var("OUTPUT_BUCKET").expect("OUTPUT_BUCKET environment variable is required");

    // fetch the audio file
    let audio_file = wget(&event.input_url).await?;

    // get an output file path
    let output_file = NamedTempFile::new()?;

    let result = visualize_audio(
        &audio_file.to_string_lossy(),
        &output_file.path().to_string_lossy(),
        preset_duration,
        &event.resolution,
    )
    .await;

    // upload the output file to S3
    let s3key = format!(
        "render/{}/{}/projectM.mp4",
        chrono::Utc::now().format("%Y-%m-%d"),
        Uuid::new_v4()
    );
    upload_to_s3(&output_file.path(), &s3_bucket, &s3key).await?;
    let output_video_url = presign_get_object(&s3_bucket, &s3key, std::time::Duration::from_secs(24 * 60 * 60)).await?;

    match result {
        Ok(_) => Ok(LambdaResponse {
            output_video_url: Some(output_video_url),
            message: "Visualization completed successfully".to_string(),
        }),
        Err(e) => {
            error!("Error during visualization: {:?}", e);
            Err(LambdaError::from(e))
        }
    }
}

async fn wget(url: &str) -> Result<PathBuf, Error> {
    // Create a temporary file
    let mut tmp_file = NamedTempFile::new()?;
    let tmp_path = tmp_file.path().to_path_buf();

    // Create a new reqwest client
    let client = Client::new();

    // Send a GET request to the URL
    let response = client.get(url).send().await?;

    // Ensure the request was successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download file: {}",
            response.status()
        ));
    }

    // Read the response bytes
    let bytes = response.bytes().await?;

    // Write the bytes to the temporary file
    tmp_file.write_all(&bytes)?;

    // Close the tempfile explicitly
    tmp_file.keep()?;

    Ok(tmp_path)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // Check if running in AWS Lambda environment
    if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        // Run the Lambda runtime
        let result = lambda_runtime::run(service_fn(lambda_handler)).await;
        if let Err(err) = result {
            error!("Error occurred: {:?}", err);
        }
    } else {
        // Retrieve arguments from environment variables
        let input_file = env::var("INPUT_AUDIO_FILE").unwrap_or_else(|_| "input.mp3".to_string());
        let output_file =
            env::var("OUTPUT_VIDEO_FILE").unwrap_or_else(|_| "output.mp4".to_string());
        let preset_duration = env::var("PRESET_DURATION")
            .unwrap_or_else(|_| DEFAULT_PRESET_DURATION.to_string())
            .parse()
            .unwrap_or(10);
        let resolution = env::var("RESOLUTION").unwrap_or_else(|_| "1920x1080".to_string());

        // Run the main function
        if let Err(err) =
            visualize_audio(&input_file, &output_file, preset_duration, &resolution).await
        {
            error!("Error occurred: {:?}", err);
        }
    }

    Ok(())
}
