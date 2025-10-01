use std::time::{Duration, Instant};
use std::thread;
use std::net::TcpStream;
use std::io;
use std::process::Command;
use reqwest::blocking::Client;
use testcontainers::{core::{WaitFor, ContainerPort}, Container, Image, ImageExt, runners::SyncRunner};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Game Server Docker Image definition
#[derive(Debug, Default, Clone)]
pub struct GameServerImage;

impl Image for GameServerImage {
    fn name(&self) -> &str {
        "numberguess-claude-rust"
    }

    fn tag(&self) -> &str {
        "latest"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Starting web server on")]
    }
}

pub struct GameServerInstance {
    pub container: Container<GameServerImage>,
    port: u16,
}

impl GameServerInstance {
    pub fn new(database_url: &str) -> Self {
        println!("Starting game server container...");

        let image = GameServerImage::default()
            .with_env_var("DATABASE_URL", database_url);

        let container = image.start().expect("Failed to start game server container");

        let port = container
            .get_host_port_ipv4(ContainerPort::Tcp(3000))
            .expect("Failed to get mapped port");

        println!("Game server container started on host port {}", port);

        // Wait for server to be ready via HTTP check
        let url = format!("http://localhost:{}", port);
        wait_for_server_ready(&url, 30)
            .expect("Server should become ready");

        println!("Game server ready at {}", url);

        Self {
            container,
            port,
        }
    }

    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    pub fn internal_url(&self) -> String {
        // Get the container's IP address on the bridge network
        let container_id = self.container.id();

        // Use docker inspect to get the container's IP address
        let output = Command::new("docker")
            .args(["inspect", "-f", "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}", container_id])
            .output()
            .expect("Failed to inspect container");

        let ip_address = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 in IP address")
            .trim()
            .to_string();

        println!("Game server container IP: {}", ip_address);

        format!("http://{}:3000", ip_address)
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
    pub container: Container<SeleniumContainer>,
    port: u16,
}

impl SeleniumInstance {
    pub fn new() -> Self {
        Self::new_with_timeout(60)
    }

    pub fn new_with_timeout(timeout_seconds: u64) -> Self {
        println!("Starting Selenium container with {}s timeout", timeout_seconds);

        let image = SeleniumContainer
            .with_shm_size(2 * 1024 * 1024 * 1024) // 2GB shared memory
            .with_env_var("SE_START_XVFB", "false")
            .with_env_var("SE_START_VNC", "false")
            .with_env_var("SE_START_NO_VNC", "false");

        let container = image.start().expect("Failed to start Selenium container");

        let port = container
            .get_host_port_ipv4(ContainerPort::Tcp(4444))
            .expect("Failed to get mapped port");

        println!("Started Selenium container, mapped port 4444 to {}", port);

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

        Self { container, port }
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

/// PostgreSQL Docker Image definition
#[derive(Debug, Default, Clone)]
pub struct PostgresImage;

impl Image for PostgresImage {
    fn name(&self) -> &str {
        "postgres"
    }

    fn tag(&self) -> &str {
        "16"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("database system is ready to accept connections")]
    }
}

pub struct PostgresInstance {
    pub container: Container<PostgresImage>,
    pub pool: PgPool,
    pub database_url: String,
}

impl PostgresInstance {
    pub fn new() -> Self {
        println!("Starting PostgreSQL container...");

        let image = PostgresImage::default()
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "postgres");

        let container = image.start().expect("Failed to start PostgreSQL container");

        let port = container
            .get_host_port_ipv4(ContainerPort::Tcp(5432))
            .expect("Failed to get mapped port");

        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            port
        );

        println!("PostgreSQL container started on host port {}", port);
        println!("Database URL: {}", database_url);

        // Create connection pool and run migrations
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let pool = runtime.block_on(async {
            // Wait for database to be fully ready with retries
            let mut retries = 0;
            let max_retries = 30;
            let pool = loop {
                thread::sleep(Duration::from_millis(500));

                match PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url)
                    .await
                {
                    Ok(pool) => {
                        println!("Successfully connected to PostgreSQL!");
                        break pool;
                    }
                    Err(e) => {
                        retries += 1;
                        if retries >= max_retries {
                            panic!("Failed to connect to database after {} retries: {:?}", max_retries, e);
                        }
                        println!("Connection attempt {} failed, retrying... ({:?})", retries, e);
                    }
                }
            };

            // Run migrations
            println!("Running migrations...");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations");

            println!("PostgreSQL instance ready!");
            pool
        });

        Self {
            container,
            pool,
            database_url,
        }
    }
}