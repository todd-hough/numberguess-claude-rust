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
        // Standard wait condition for Selenium Grid
        // The version of testcontainers we're using doesn't support custom wait functions
        // so we'll stick with the message-based check and supplement with our own checks
        vec![WaitFor::message_on_stdout("Selenium Grid ready")]
    }
}

pub struct SeleniumInstance {
    container: Container<SeleniumContainer>,
    port: u16,
    timeout_seconds: u64,
}

impl SeleniumInstance {
    pub fn new() -> Self {
        Self::new_with_timeout(60) // Default to 60 seconds timeout
    }
    
    pub fn new_with_timeout(timeout_seconds: u64) -> Self {
        // Start Selenium container with a random port
        let host_port = find_available_port();
        let container_port = 4444;
        
        println!("Starting Selenium container with {}s timeout", timeout_seconds);
        
        // Start the container - we'll handle the wait logic manually
        let mut retries = 3;
        let mut container;
        let mut port;
        
        loop {
            match SeleniumContainer
                .with_mapped_port(host_port, ContainerPort::Tcp(container_port))
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
            
        Self { container, port, timeout_seconds }
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