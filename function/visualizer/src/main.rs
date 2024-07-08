use std::env;
use std::io::Write;
use std::path::PathBuf;
use anyhow::Error;
use env_logger;
use lambda_runtime::{Error as LambdaError, LambdaEvent, service_fn};
use log::{error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use projectm_lambda::{lambda_handler, visualize_audio};

// how long each preset should be shown in the visualization
const DEFAULT_PRESET_DURATION: u32 = 5;

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
        let output_file = env::var("OUTPUT_VIDEO_FILE").unwrap_or_else(|_| "output.mp4".to_string());
        let preset_duration = env::var("PRESET_DURATION").unwrap_or_else(|_| DEFAULT_PRESET_DURATION.to_string()).parse().unwrap_or(10);
        let resolution = env::var("RESOLUTION").unwrap_or_else(|_| "1920x1080".to_string());

        // Run the main function
        if let Err(err) = projectm_lambda::visualize_audio(&input_file, &output_file, preset_duration, &resolution).await {
            error!("Error occurred: {:?}", err);
        }
    }

    Ok(())
}

