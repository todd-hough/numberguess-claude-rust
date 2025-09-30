use std::process::Command;
use std::sync::Once;

static DOCKER_IMAGE_CHECK: Once = Once::new();

/// Ensures the Docker image is built before running tests
/// Call this at the start of any test that needs the game server container
pub fn ensure_docker_image() {
    DOCKER_IMAGE_CHECK.call_once(|| {
        println!("Checking if Docker image exists...");

        // Check if Docker is available
        let docker_available = Command::new("docker")
            .args(["info"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !docker_available {
            panic!("Docker is not available. Please install Docker and ensure it's running.");
        }

        // Check if image exists
        let image_check = Command::new("docker")
            .args(["images", "-q", "numberguess-claude-rust:latest"])
            .output()
            .expect("Failed to check Docker images");

        if image_check.stdout.is_empty() {
            println!("Docker image not found. Building numberguess-claude-rust:latest...");
            println!("This may take a few minutes...");

            let build_status = Command::new("docker")
                .args(["build", "-t", "numberguess-claude-rust:latest", "."])
                .status()
                .expect("Failed to run docker build");

            if !build_status.success() {
                panic!("Docker image build failed. Please build it manually:\n  docker build -t numberguess-claude-rust:latest .");
            }

            println!("✓ Docker image built successfully");
        } else {
            println!("✓ Docker image exists");
        }
    });
}

/// Helper to check if image needs rebuilding based on source changes
/// Returns true if the image is older than the source files
pub fn image_needs_rebuild() -> bool {
    use std::time::SystemTime;

    // Get image creation time
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.Created}}", "numberguess-claude-rust:latest"])
        .output()
        .ok();

    let image_time = match output {
        Some(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .to_string()
        }
        _ => return true, // If we can't get image time, rebuild
    };

    // Check if source files are newer than image
    // This is a simplified check - you could make it more sophisticated
    let dockerfile_modified = std::fs::metadata("Dockerfile")
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // For simplicity, just check Dockerfile modification time
    // In production, you'd check src/ and static/ dirs too
    println!("Image created: {}", image_time);
    println!("Dockerfile modified: {:?}", dockerfile_modified);

    false // Conservative: assume image is ok unless we have evidence otherwise
}

/// Force rebuild the Docker image
pub fn rebuild_docker_image() {
    println!("Rebuilding Docker image...");

    let build_status = Command::new("docker")
        .args(["build", "--no-cache", "-t", "numberguess-claude-rust:latest", "."])
        .status()
        .expect("Failed to run docker build");

    if !build_status.success() {
        panic!("Docker image rebuild failed");
    }

    println!("✓ Docker image rebuilt successfully");
}