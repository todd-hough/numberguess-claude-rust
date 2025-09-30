use std::process::Command;

fn main() {
    // Only build Docker image for integration tests
    // Check if we're building for tests
    let profile = std::env::var("PROFILE").unwrap_or_default();

    println!("cargo:rerun-if-changed=Dockerfile");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=static/");

    // Only build image when running tests
    if std::env::var("CARGO_CFG_TEST").is_ok() || profile == "test" {
        println!("cargo:warning=Checking Docker image for tests...");

        // Check if Docker is available
        let docker_available = Command::new("docker")
            .args(["info"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !docker_available {
            println!("cargo:warning=Docker not available - skipping image build");
            return;
        }

        // Check if image exists and is recent
        let image_exists = Command::new("docker")
            .args(["images", "-q", "numberguess-claude-rust:latest"])
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false);

        if image_exists {
            println!("cargo:warning=Docker image exists - skipping build");
            return;
        }

        println!("cargo:warning=Building Docker image for tests (this may take a few minutes)...");

        // Build the Docker image
        let status = Command::new("docker")
            .args(["build", "-t", "numberguess-claude-rust:latest", "."])
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("cargo:warning=Docker image built successfully");
            }
            Ok(status) => {
                eprintln!("cargo:warning=Docker build failed with status: {}", status);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("cargo:warning=Failed to run docker build: {}", e);
                std::process::exit(1);
            }
        }
    }
}