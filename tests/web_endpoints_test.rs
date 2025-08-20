mod common;

use common::containers::GameServerInstance;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GameResponse {
    game_id: u64,
    min: u32,
    max: u32,
    max_guesses: Option<u32>,
    message: String,
}

#[test]
fn test_static_file_serving() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Test root serves index.html
    let resp = client
        .get(server.url())
        .send()
        .unwrap();
    
    assert!(resp.status().is_success(), "Root URL should return successful response");
    let body = resp.text().unwrap();
    assert!(body.contains("Number Guessing Game"), "Response should contain game title");
    assert!(body.contains("<!DOCTYPE html>"), "Response should be HTML");
    
    println!("✅ Static file serving test passed");
}

#[test]
fn test_web_form_endpoints() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Test form submission to /game/new
    let resp = client
        .post(format!("{}/game/new", server.url()))
        .form(&[("min", "1"), ("max", "10"), ("max_guesses", "5")])
        .send()
        .unwrap();
    
    assert!(resp.status().is_success(), "Form submission should succeed");
    let body = resp.text().unwrap();
    
    // Should return HTML with game interface
    assert!(body.contains("guess"), "Response should contain game interface");
    assert!(body.contains("form"), "Response should contain form element");
    
    println!("✅ Web form endpoints test passed");
}