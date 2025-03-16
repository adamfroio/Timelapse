use std::process::Command;

fn main() {
    let output = Command::new("libcamera-still") // Replace with "dir" on Windows
        .arg("-o") // Add arguments if needed
        .arg("/home/afroio/Desktop/test001.jpg")
        .output() // Execute and capture output
        .expect("Failed to execute command");

    // Convert and print the output
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
