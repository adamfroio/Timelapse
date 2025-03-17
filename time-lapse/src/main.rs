use std::env;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::Read;
use reqwest;
use serde::Deserialize;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use oauth2::{RedirectUrl, Scope};
use reqwest::blocking::multipart;
use ffmpeg_next::format::{input, output};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your latitude & longitude
    let latitude = 37.7749;
    let longitude = -122.4194;
    
    let url = format!(
        "https://api.sunrise-sunset.org/json?lat={}&lng={}&formatted=0",
        latitude, longitude
    );

    // Send request
    let response: ApiResponse = reqwest::get(&url).await?.json().await?;

    // Print sunset time (UTC)
    println!("Sunrise time (UTC): {}", response.results.sunrise);
    println!("Sunset time (UTC): {}", response.results.sunset);


    // upload_video("YOUR_ACCESS_TOKEN", "path/to/video.mp4").unwrap();

    Ok(())
}

#[derive(Deserialize)]
struct ApiResponse {
    results: SunTimes,
}

#[derive(Deserialize)]
struct SunTimes {
    sunrise: String,
    sunset: String,
}


fn capture_images() {
    let interval = Duration::from_secs(30); // Capture an image every 30 seconds
    let output_dir = "/home/afroio/Desktop/timelapse_images";

    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(output_dir).expect("Failed to create directory");

    for i in 0..100 { // Capture 100 images
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let filename = format!("{}/img_{}.jpg", output_dir, timestamp);

        let status = Command::new("libcamera-still")
            .args(["-o", &filename, "--width", "1920", "--height", "1080", "--timeout", "1000"])
            .status()
            .expect("Failed to execute libcamera-still");

        if status.success() {
            println!("Captured image: {}", filename);
        } else {
            eprintln!("Failed to capture image");
        }

        sleep(interval);
    }
}


fn create_timelapse() {
    let status = Command::new("ffmpeg")
        .args(["-framerate", "10", "-pattern_type", "glob", "-i", "timelapse_images/*.jpg", "-c:v", "libx264", "-pix_fmt", "yuv420p", "timelapse.mp4"])
        .status()
        .expect("Failed to execute ffmpeg");
    
    if status.success() {
        println!("Timelapse video created: timelapse.mp4");
    } else {
        eprintln!("Failed to create timelapse video");
    }
}




fn get_oauth_client() -> BasicClient {
    let client_id = ClientId::new(env::var("YOUTUBE_CLIENT_ID").expect("Missing CLIENT_ID"));
    let client_secret = ClientSecret::new(env::var("YOUTUBE_CLIENT_SECRET").expect("Missing CLIENT_SECRET"));

    BasicClient::new(
        client_id,
        Some(client_secret),
        AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string()).unwrap())
}

fn upload_video(access_token: &str, video_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.googleapis.com/upload/youtube/v3/videos?uploadType=resumable&part=snippet,status";

    // Video metadata
    let metadata = serde_json::json!({
        "snippet": {
            "title": "My Rust Video",
            "description": "Uploaded via Rust",
            "tags": ["rust", "api"],
            "categoryId": "22" // 22 = People & Blogs
        },
        "status": {
            "privacyStatus": "public"
        }
    });

    // Create client
    let client = reqwest::blocking::Client::new();

    // Step 1: Initiate upload
    let response = client
        .post(url)
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .header("X-Upload-Content-Type", "video/mp4")
        .header("X-Upload-Content-Length", fs::metadata(video_path)?.len().to_string())
        .json(&metadata)
        .send()?;

    let upload_url = response
        .headers()
        .get("Location")
        .ok_or("Missing Upload URL")?
        .to_str()?
        .to_string();

    // Step 2: Upload the file
    let mut file = File::open(video_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    client
        .put(&upload_url)
        .bearer_auth(access_token)
        .header("Content-Type", "video/mp4")
        .body(buffer)
        .send()?;

    println!("Video uploaded successfully!");

    Ok(())
}

