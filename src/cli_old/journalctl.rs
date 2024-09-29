
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Flag to indicate whether to keep running
    let running = Arc::new(AtomicBool::new(true));

    // Handle Ctrl+C (SIGINT) to stop the loop
    let r = Arc::clone(&running);
    ctrlc::set_handler(move || {
        println!("Ctrl+C pressed. Exiting...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Start the `tail -f /var/log/syslog` command
    let mut child = Command::new("tail")
        .arg("-f")
        .arg("/var/log/syslog")  // Replace with your desired file
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start `tail` command");

    // Ensure we can read from the child process's `stdout`
    let stdout = child.stdout.as_mut().expect("Failed to open `stdout`");

    // Use a buffered reader to read the lines from the output
    let reader = BufReader::new(stdout);

    // Loop to continuously read lines from the `tail` command output
    for line in reader.lines() {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        match line {
            Ok(line) => {
                // Process each line here
                println!("Read line: {}", line);
                // Example processing: Filter, transform, etc.
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        }
    }

    // Terminate the child process when done
    let _ = child.kill();
    let _ = child.wait();
}
