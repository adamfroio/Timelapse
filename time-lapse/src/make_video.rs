use gstreamer as gst;
use gst::prelude::*;
use gst::parse;

fn main()
{
 // Initialize GStreamer
    gst::init().expect("Failed to initialize GStreamer");

	   let image_path = "/home/afroio/Desktop/timelapse_images/img_%03d.jpg";


    // Create a pipeline
    let pipeline = parse::launch(&format!(
        "multifilesrc location={} index=1 \
         ! image/jpeg,framerate=30/1 \
         ! jpegparse ! jpegdec ! videoconvert ! x264enc \
         ! mp4mux ! filesink location=output.mp4",
         image_path
    ))
    .expect("Failed to create pipeline");

    let bus = pipeline.bus().expect("Couldn't get pipeline bus");
    pipeline.set_state(gst::State::Playing).expect("Failed to start pipeline");

    // Listen for the end of the process
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        match msg.view() {
            gst::MessageView::Eos(..) => break,
            gst::MessageView::Error(err) => {
                eprintln!(
                    "Error from {}: {} ({:?})",
                    err.src().map(|s| s.path_string()).unwrap_or_else(|| "Unknown".into()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => {}
        }
    }

    // Stop pipeline
    pipeline.set_state(gst::State::Null).expect("Failed to stop pipeline");
}
