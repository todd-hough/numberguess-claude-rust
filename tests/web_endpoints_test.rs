mod common;

use common::containers::{GameServerInstance, PostgresInstance};
use reqwest::blocking::Client;

#[test]
fn test_static_file_serving() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let client = Client::new();
    
    // Test root serves index.html
    let resp = client
        .get(server.url())
        .send()
        .expect("Should send GET request to root URL");

    assert!(resp.status().is_success(), "Root URL should return successful response");
    let body = resp.text().expect("Should get response body as text");
    assert!(body.contains("Number Guessing Game"), "Response should contain game title");
    assert!(body.contains("<!DOCTYPE html>"), "Response should be HTML");
    
    println!("✅ Static file serving test passed");
}

#[test]
fn test_web_form_endpoints() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let client = Client::new();
    
    // Test form submission to /game/new
    let resp = client
        .post(format!("{}/game/new", server.url()))
        .form(&[("min", "1"), ("max", "10"), ("max_guesses", "5")])
        .send()
        .expect("Should send POST request with form data");

    assert!(resp.status().is_success(), "Form submission should succeed");
    let body = resp.text().expect("Should get response body as text");
    
    // Should return HTML with game interface
    assert!(body.contains("guess"), "Response should contain game interface");
    assert!(body.contains("form"), "Response should contain form element");
    
    println!("✅ Web form endpoints test passed");
}