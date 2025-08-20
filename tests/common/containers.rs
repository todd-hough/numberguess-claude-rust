use std::time::{Duration, Instant};
use std::thread;
use std::process::{Command, Child};
use reqwest::blocking::Client;

pub struct GameServerInstance {
    process: Child,
    port: u16,
}

impl GameServerInstance {
    pub fn new(port: u16) -> Self {
        println!("Starting game server on port {}", port);
        
        let process = Command::new("cargo")
            .args(["run", "--", "--server", "--port", &port.to_string()])
            .spawn()
            .expect("Failed to start game server");
            
        let instance = Self { process, port };
        
        // Wait for server to be ready
        let url = format!("http://localhost:{}", port);
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