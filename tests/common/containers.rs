use std::time::{Duration, Instant};
use std::thread;
use std::process::{Command, Child};
use std::net::{TcpListener, SocketAddrV4, Ipv4Addr, TcpStream};
use std::io;
use reqwest::blocking::Client;
use rand::Rng;
use testcontainers::{core::{WaitFor, ContainerPort}, Container, Image, ImageExt, runners::SyncRunner};

/// Find an available port in the ephemeral port range (49152-65535)
pub fn find_available_port() -> u16 {
    // Try up to 10 times to find an available port
    for _ in 0..10 {
        // Generate a random port in the ephemeral port range (49152-65535)
        let port = rand::rng().random_range(49152..=65535);
        
        // Check if the port is available
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
        if TcpListener::bind(addr).is_ok() {
            return port;
        }
    }
    
    // If we can't find a port after multiple attempts, use a fallback port
    // This is unlikely to happen but provides a fallback
    eprintln!("Warning: Couldn't find available port, using fallback");
    49152
}

pub struct GameServerInstance {
    process: Child,
    port: u16,
}

impl GameServerInstance {
    pub fn new() -> Self {
        // Get a random available port
        let port = find_available_port();
        println!("Starting game server on port {}", port);
        
        let process = Command::new("cargo")
            .args(["run", "--", "--server", "--port", &port.to_string()])
            .spawn()
            .expect("Failed to start game server");
            
        let instance = Self { process, port };
        
        // Wait for server to be ready
        let url = instance.url();
        wait_for_server_ready(&url, 30)
            .expect("Server should become ready");
            
        println!("Server ready on port {}", port);
        
        instance
    }
    
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

impl Drop for GameServerInstance {
    fn drop(&mut self) {
        println!("Stopping game server on port {}", self.port);
        
        if let Err(e) = self.process.kill() {
            eprintln!("Failed to kill game server: {}", e);
        }
    }
}


pub fn wait_for_server_ready(url: &str, max_seconds: u64) -> Result<(), String> {
    let client = Client::new();
    let start = Instant::now();
    let max_duration = Duration::from_secs(max_seconds);
    
    while start.elapsed() < max_duration {
        match client.get(&format!("{}/", url)).send() {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    Err(format!("Server at {} not ready after {} seconds", url, max_seconds))
}

/// A struct representing a Selenium Chrome container
pub struct SeleniumContainer;

impl Image for SeleniumContainer {
    fn name(&self) -> &str {
        "seleniarm/standalone-chromium"
    }
    fn tag(&self) -> &str {
        "latest"
    }
    fn ready_conditions(&self) -> Vec<WaitFor> {
        // Wait for Selenium to be ready - this matches what appears in the logs
        // when Selenium container starts successfully
        vec![WaitFor::message_on_stdout("Started Selenium Standalone")]
    }
}

pub struct SeleniumInstance {
    container: Container<SeleniumContainer>,
    port: u16,
    timeout_seconds: u64,
    game_server_host: String,
}

impl SeleniumInstance {
    pub fn new() -> Self {
        Self::new_with_timeout(60) // Default to 60 seconds timeout
    }
    
    pub fn new_with_game_server(game_server_port: u16, timeout_seconds: u64) -> Self {
        // Determine the host address that the container should use to reach the game server
        // For Linux, we need to get the host's IP address that's accessible from Docker
        let game_server_host = Self::get_host_address(game_server_port);
        
        println!("Starting Selenium container with game server at {}", game_server_host);
        
        // Start Selenium container with access to host network
        let host_port = find_available_port();
        let container_port = 4444;
        
        println!("Starting Selenium container with {}s timeout", timeout_seconds);
        
        let mut retries = 3;
        let container;
        let port;
        
        loop {
            match SeleniumContainer
                .with_mapped_port(host_port, ContainerPort::Tcp(container_port))
                .with_shm_size(2 * 1024 * 1024 * 1024) // 2GB shared memory
                .with_env_var("SE_START_XVFB", "false")
                .with_env_var("SE_START_VNC", "false")
                .with_env_var("SE_START_NO_VNC", "false")
                .start() {
                    Ok(c) => {
                        container = c;
                        port = container
                            .get_host_port_ipv4(ContainerPort::Tcp(container_port))
                            .expect("Failed to get mapped port");
                        println!("Started Selenium container, mapped port {} to {}", container_port, port);
                        break;
                    },
                    Err(e) => {
                        if retries > 0 {
                            println!("Failed to start container: {}. Retrying...", e);
                            retries -= 1;
                            thread::sleep(Duration::from_secs(1));
                        } else {
                            panic!("Failed to start Selenium container after multiple attempts: {}", e);
                        }
                    }
                }
        }
        
        let url = format!("http://localhost:{}", port);
        println!("Container started, waiting for Selenium to be ready at {}", url);
        
        match wait_for_selenium_ready(&url, timeout_seconds) {
            Ok(_) => println!("Selenium is ready!"),
            Err(e) => {
                println!("Selenium HTTP endpoint not ready: {}", e);
                println!("Falling back to port binding verification...");
                if verify_port_binding("localhost", port, timeout_seconds / 3).is_ok() {
                    println!("Port binding verification successful - proceeding anyway");
                } else {
                    panic!("All readiness checks failed - Selenium is not responding");
                }
            }
        }
            
        Self { container, port, timeout_seconds, game_server_host }
    }
    
    fn get_host_address(port: u16) -> String {
        // On Linux, we need to get the Docker bridge IP
        // Try to get the docker0 interface IP or fallback to host.docker.internal
        if let Ok(output) = Command::new("ip")
            .args(["route", "show", "default"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Look for the default gateway which is usually the Docker host from container perspective  
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 2 {
                        // Try getting the docker0 bridge IP instead
                        if let Ok(docker_ip_output) = Command::new("ip")
                            .args(["-4", "addr", "show", "docker0"])
                            .output() {
                            if let Ok(docker_stdout) = String::from_utf8(docker_ip_output.stdout) {
                                for line in docker_stdout.lines() {
                                    if line.contains("inet ") {
                                        let parts: Vec<&str> = line.split_whitespace().collect();
                                        if parts.len() > 1 {
                                            if let Some(ip) = parts[1].split('/').next() {
                                                println!("Using Docker bridge IP: {}", ip);
                                                return format!("http://{}:{}", ip, port);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: try host.docker.internal (works on Docker Desktop)
        // or use 172.17.0.1 which is the default Docker bridge gateway
        println!("Using default Docker host address");
        format!("http://172.17.0.1:{}", port)
    }
    
    pub fn game_server_url(&self) -> String {
        self.game_server_host.clone()
    }
    
    pub fn new_with_timeout(timeout_seconds: u64) -> Self {
        // Start Selenium container with a random port
        let host_port = find_available_port();
        let container_port = 4444;
        
        println!("Starting Selenium container with {}s timeout", timeout_seconds);
        
        // Start the container - we'll handle the wait logic manually
        let mut retries = 3;
        let container;
        let port;
        
        loop {
            match SeleniumContainer
                .with_mapped_port(host_port, ContainerPort::Tcp(container_port))
                .with_shm_size(2 * 1024 * 1024 * 1024) // 2GB shared memory like --shm-size 2g
                .with_env_var("SE_START_XVFB", "false") // Disable Xvfb since we're running headless
                .with_env_var("SE_START_VNC", "false") // Disable VNC to reduce resource usage
                .with_env_var("SE_START_NO_VNC", "false") // Disable noVNC web interface
                .start() {
                    Ok(c) => {
                        container = c;
                        
                        // Get the mapped port assigned by Docker
                        port = container
                            .get_host_port_ipv4(ContainerPort::Tcp(container_port))
                            .expect("Failed to get mapped port");
                            
                        println!("Started Selenium container, mapped port {} to {}", container_port, port);
                        break;
                    },
                    Err(e) => {
                        if retries > 0 {
                            println!("Failed to start container: {}. Retrying...", e);
                            retries -= 1;
                            thread::sleep(Duration::from_secs(1));
                        } else {
                            panic!("Failed to start Selenium container after multiple attempts: {}", e);
                        }
                    }
                }
        }
        
        // Wait for Selenium to be ready with our manual HTTP check
        let url = format!("http://localhost:{}", port);
        println!("Container started, waiting for Selenium to be ready at {}", url);
        
        // Use our HTTP readiness check with extended timeout
        match wait_for_selenium_ready(&url, timeout_seconds) {
            Ok(_) => println!("Selenium is ready!"),
            Err(e) => {
                println!("Selenium HTTP endpoint not ready: {}", e);
                
                // Fall back to port binding verification
                println!("Falling back to port binding verification...");
                if verify_port_binding("localhost", port, timeout_seconds / 3).is_ok() {
                    println!("Port binding verification successful - proceeding anyway");
                } else {
                    panic!("All readiness checks failed - Selenium is not responding");
                }
            }
        }
            
        Self { container, port, timeout_seconds, game_server_host: format!("http://localhost:{}", port) }
    }
    
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

pub fn wait_for_selenium_ready(url: &str, max_seconds: u64) -> Result<(), String> {
    let client = Client::new();
    let start = Instant::now();
    let max_duration = Duration::from_secs(max_seconds);
    
    // Selenium status endpoint
    let status_url = format!("{}/status", url);
    println!("Checking if Selenium is ready at: {}", status_url);
    
    while start.elapsed() < max_duration {
        match client.get(&status_url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Selenium HTTP endpoint is ready!");
                    return Ok(());
                } else {
                    println!("HTTP status: {}", response.status());
                }
            },
            Err(e) => {
                println!("HTTP error: {}", e);
            }
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    Err(format!("Selenium at {} not ready after {} seconds", url, max_seconds))
}

// Simple helper function to check if an HTTP endpoint is ready
pub fn check_http_endpoint(url: &str) -> bool {
    let client = Client::new();
    
    match client.get(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false
    }
}

// Port binding verification as a last-resort fallback
pub fn verify_port_binding(host: &str, port: u16, timeout_seconds: u64) -> Result<(), String> {
    println!("Checking port binding for {}:{}", host, port);
    let start = Instant::now();
    let max_duration = Duration::from_secs(timeout_seconds);
    
    while start.elapsed() < max_duration {
        match TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(_) => {
                println!("Successfully connected to {}:{}", host, port);
                return Ok(());
            },
            Err(e) => {
                if e.kind() != io::ErrorKind::ConnectionRefused {
                    println!("Connection error: {}", e);
                }
            }
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    Err(format!("Port {}:{} not available after {} seconds", host, port, timeout_seconds))
}