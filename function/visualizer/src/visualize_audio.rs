use anyhow::{anyhow, Error};
use gstreamer as gst;
use gstreamer::prelude::*;
use log::{debug, error, info, warn};
use regex::Regex;
use std::env;
use std::time::{Duration, Instant};

pub async fn visualize_audio(
    input_file: &str,
    output_file: &str,
    preset_duration: u32,
    resolution: &str,
) -> Result<(), Error> {
    // Initialize GStreamer
    gst::init()?;

    // get from environment variable or panic
    let presets_dir =
        env::var("PRESETS_DIR").expect("PRESETS_DIR environment variable is required");

    // Validate and parse the resolution
    let resolution_regex = Regex::new(r"^(\d+)x(\d+)$").unwrap();
    let captures = resolution_regex
        .captures(resolution)
        .ok_or_else(|| anyhow!("Invalid resolution format"))?;
    let width: u32 = captures
        .get(1)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap_or(1920);
    let height: u32 = captures
        .get(2)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap_or(1080);

    // calculate mesh x/y size based on resolution
    let mesh_x = (width as f32 / 30.0).ceil() as u32;
    let mesh_y = (height as f32 / 30.0).ceil() as u32;
    let mesh_size = format!("{},{}", mesh_x, mesh_y);

    info!("Input audio file: {}", input_file);
    info!("Output video file: {}", output_file);
    info!("Resolution: {}x{}", width, height);
    info!("Mesh size: {}", mesh_size);
    info!("Preset duration: {}", preset_duration);

    // Build the pipeline
    let pipeline_str = format!(
        "filesrc location={} ! decodebin name=dec \
    dec. ! queue ! audioconvert ! audioresample ! tee name=t \
    t. ! queue ! avenc_aac bitrate=128000 ! queue ! mux. \
    t. ! queue ! audioconvert ! projectm preset={} preset-duration={} mesh-size={} ! videoconvert ! video/x-raw,width={},height={},framerate=30/1 ! x264enc bitrate=5000 ! h264parse ! mp4mux name=mux ! filesink location={}",
        input_file, presets_dir, preset_duration, mesh_size, width, height, output_file
    );

    let pipeline = gst::parse::launch(&pipeline_str)?;

    // Log the constructed pipeline
    debug!("Pipeline: {:?}", pipeline);

    // Start playing
    info!("Starting the pipeline");
    match pipeline.set_state(gst::State::Playing) {
        Ok(_) => info!("Pipeline is now playing"),
        Err(err) => {
            error!(
                "Unable to set the pipeline to the `Playing` state: {:?}",
                err
            );
            return Err(err.into());
        }
    }

    // Wait until error or EOS
    let bus = pipeline.bus().unwrap();
    let mut eos_received = false;
    let start_time = Instant::now();

    // Timer for progress indicator
    let mut last_progress_time = Instant::now();
    let mut frame_count = 0;

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        // Progress indicator logic
        if last_progress_time.elapsed() >= Duration::from_secs(10) {
            let elapsed = start_time.elapsed().as_secs();
            let duration = pipeline.query_duration::<gst::ClockTime>().unwrap_or(gst::ClockTime::ZERO);
            let percent_done = if duration != gst::ClockTime::ZERO {
                (elapsed * 100 / duration.seconds()).min(100)
            } else {
                0
            };

            // Calculate the processing rate (fps)
            let fps = frame_count as f64 / last_progress_time.elapsed().as_secs_f64();
            frame_count = 0;

            info!(
                "Progress: {}% done at timestamp: {:?}, processing rate: {:.2} fps",
                percent_done, start_time + Duration::from_secs(elapsed), fps
            );
            last_progress_time = Instant::now();
        }

        match msg.view() {
            MessageView::Eos(..) => {
                info!("End of stream reached");
                eos_received = true;
                break;
            }
            MessageView::Error(err) => {
                error!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            MessageView::StateChanged(state_changed) => {
                if let Some(element) = state_changed.src() {
                    debug!(
                        "State changed in element {:?} from {:?} to {:?}",
                        element.path_string(),
                        state_changed.old(),
                        state_changed.current()
                    );

                    if element == pipeline.dynamic_cast_ref::<gst::Element>().unwrap()
                        && state_changed.current() == gst::State::Paused
                    {
                        warn!("Pipeline is paused unexpectedly, checking further...");
                        pipeline.set_state(gst::State::Playing)?;
                    }
                }
            }
            MessageView::Buffering(buffering) => {
                info!("Buffering {}%", buffering.percent());
                if buffering.percent() < 100 {
                    info!("Pipeline is buffering, pausing");
                    pipeline.set_state(gst::State::Paused)?;
                } else {
                    info!("Buffering complete, resuming playback");
                    pipeline.set_state(gst::State::Playing)?;
                }
            }
            MessageView::Latency(..) => {
                info!("Latency updated");
                if let Some(bin) = pipeline.dynamic_cast_ref::<gst::Bin>() {
                    bin.recalculate_latency().unwrap();
                }
            }
            MessageView::StreamStatus(status) => {
                debug!("Stream status changed: {:?}", status);
            }
            MessageView::DurationChanged(..) => {
                info!("Duration changed");
            }
            MessageView::ClockLost(..) => {
                warn!("Clock lost, setting state to Playing");
                pipeline.set_state(gst::State::Playing).unwrap();
            }
            _ => {
                debug!("Received message: {:?}", msg);
            }
        }

        // Increment the frame count for processing rate calculation
        frame_count += 1;
    }

    if !eos_received {
        warn!("No EOS received, pipeline may not have completed");
    }

    // Shutdown pipeline
    info!("Shutting down the pipeline");
    match pipeline.set_state(gst::State::Null) {
        Ok(_) => info!("Pipeline is now null"),
        Err(err) => {
            error!("Unable to set the pipeline to the `Null` state: {:?}", err);
        }
    }

    Ok(())
}
