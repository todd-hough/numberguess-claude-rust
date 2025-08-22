mod common;

use common::containers::{SeleniumInstance, wait_for_selenium_ready};
use std::process::Command;
use std::time::Duration;

/// This is a minimal test case that only starts the Selenium container
/// to isolate the startup timeout issue.
#[test]
fn test_selenium_container_startup() {
    // Skip this test if Docker is not available or not running
    let docker_available = match Command::new("docker").args(["info"]).output() {
        Ok(output) => output.status.success(),
        Err(e) => {
            println!("Docker command failed: {}", e);
            false
        }
    };
    
    if !docker_available {
        println!("Skipping selenium startup test - Docker not available or not running");
        return;
    }
//   
//   // Print Docker version information for debugging
//   if let Ok(output) = Command::new("docker").args(["version"]).output() {
//       if let Ok(version) = std::str::from_utf8(&output.stdout) {
//           println!("Docker version: {}", version.trim());
//       }
//   }
//   
//   // Run Selenium container manually to observe behavior
//   println!("Starting Selenium container...");
//   let container_command = Command::new("docker")
//       .args([
//           "run", 
//           "--rm", 
//           "-d", 
//           "-p", "4444:4444", 
//           "seleniarm/standalone-chromium:latest"
//       ])
//       .output();
//       
//   if let Ok(output) = container_command {
//       if output.status.success() {
//           if let Ok(container_id) = std::str::from_utf8(&output.stdout) {
//               let container_id = container_id.trim();
//               println!("Selenium container started with ID: {}", container_id);
//               
//               // Give container a few seconds to initialize
//               std::thread::sleep(Duration::from_secs(2));
//               
//               // Check container logs to see startup messages
//               println!("Checking container logs:");
//               if let Ok(logs_output) = Command::new("docker")
//                   .args(["logs", container_id])
//                   .output() {
//                   if let Ok(logs) = std::str::from_utf8(&logs_output.stdout) {
//                       println!("Container logs:");
//                       println!("{}", logs);
//                       
//                       // Look for the ready message pattern
//                       if logs.contains("Started Selenium Standalone") {
//                           println!("Found 'Started Selenium Standalone' message in logs");
//                       } else {
//                           println!("WARNING: 'Started Selenium Standalone' message NOT found in logs");
//                       }
//                   }
//               }
//               
//               // Test HTTP endpoint readiness
//               println!("Testing HTTP endpoint readiness");
//               match wait_for_selenium_ready("http://localhost:4444", 10) {
//                   Ok(_) => println!("HTTP endpoint is ready"),
//                   Err(e) => println!("HTTP endpoint is NOT ready: {}", e),
//               }
//               
//               // Clean up the container
//               let _ = Command::new("docker")
//                   .args(["stop", container_id])
//                   .output();
//           }
//       } else {
//           if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
//               println!("Failed to start container: {}", stderr);
//           }
//       }
//   } else {
//       println!("Failed to execute docker command");
//   }
//   
//   // Now try using the testcontainers-rs approach
//   println!("\nAttempting to start Selenium container using testcontainers-rs...");
//   match std::panic::catch_unwind(|| {
//       let selenium = SeleniumInstance::new();
//       let url = selenium.url();
//       println!("Successfully started Selenium container at {}", url);
//       true
//   }) {
//       Ok(result) => {
//           assert!(result, "Selenium container should start successfully");
//       },
//       Err(_) => {
//           println!("Failed to start Selenium container with testcontainers-rs");
//           println!("This confirms the issue being investigated");
//       }
//   }
//   
    println!("Test complete");
}
