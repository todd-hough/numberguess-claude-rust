use std::time::{Duration, Instant};
use std::thread;
use std::process::{Command, Child};
use std::net::{TcpListener, SocketAddrV4, Ipv4Addr};
use reqwest::blocking::Client;
use rand::Rng;

/// Find an available port in the ephemeral port range (49152-65535)
pub fn find_available_port() -> u16 {
    // Try up to 10 times to find an available port
    for _ in 0..10 {
        // Generate a random port in the ephemeral port range (49152-65535)
        let port = rand::thread_rng().gen_range(49152..=65535);
        
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
    
    pub fn port(&self) -> u16 {
        self.port
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