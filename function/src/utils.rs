use anyhow::Error;
use reqwest::Client;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use aws_sdk_s3::{Client as S3Client};
use std::path::Path;
use std::time::Duration;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use glutin::context;
use gstreamer_gl as gst_gl;
use gstreamer as gst;

pub async fn wget(url: &str) -> Result<PathBuf, Error> {
    // Create a temporary file
    let mut tmp_file = NamedTempFile::new()?;
    let tmp_path = tmp_file.path().to_path_buf();

    // Create a new reqwest client
    let client = Client::new();

    // Send a GET request to the URL
    let response = client.get(url).send().await?;

    // Ensure the request was successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download file: {}", response.status()));
    }

    // Read the response bytes
    let bytes = response.bytes().await?;

    // Write the bytes to the temporary file
    let mut file = tokio::fs::File::from_std(tmp_file.reopen()?);
    file.write_all(&bytes).await?;

    // Close the tempfile explicitly
    drop(file);
    Ok(tmp_path)
}


pub async fn upload_to_s3(file_path: &Path, bucket: &str, key: &str) -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = S3Client::new(&config);

    let body = ByteStream::from_path(file_path).await?;

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(())
}

pub async fn presign_get_object(bucket: &str, key: &str, expiration: Duration) -> Result<String, Error> {
    let config = aws_config::load_from_env().await;
    let client = S3Client::new(&config);

    let presign_config = PresigningConfig::expires_in(expiration)?;

    let presigned_req = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presign_config)
        .await?;

    Ok(presigned_req.uri().to_string())
}

// pub fn initialize_gl_context() -> Result<WindowedContext<PossiblyCurrent>, Error> {
//     // let el = glutin::event_loop::EventLoop::new();
//     // let wb = glutin::window::WindowBuilder::new().with_visible(false);
//     // let windowed_context = ContextBuilder::new()
//     //     .with_gl(glutin::GlRequest::Latest)
//     //     .with_gl_profile(glutin::GlProfile::Core)
//     //     .with_gl_debug_flag(true)
//     //     .build_windowed(wb, &el)
//     //     .map_err(|e| anyhow::anyhow!("Failed to create windowed context: {:?}", e))?;
//     // let windowed_context = unsafe { windowed_context.make_current().map_err(|(_, e)| anyhow::anyhow!("Failed to make context current: {:?}", e))? };
//     Ok(windowed_context)
// }
//
// pub fn get_gl_display_and_context() -> Result<(gst_gl::GLDisplay, gst::gl::GLContext), Error> {
//     let windowed_context = initialize_gl_context()?;
//     let display = gst_gl::GLDisplayEGL::new();
//     let context = unsafe { gst_gl::GLContext::new_wrapped(&display, windowed_context.context(), gst_gl::GLPlatform::EGL, gst_gl::GLAPI::GLES2, None)? };
//     Ok((display, context))
// }