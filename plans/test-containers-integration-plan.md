# Test Containers Integration Testing Plan

## Executive Summary

This document outlines a comprehensive plan to introduce automated integration tests using test containers for the Number Guessing Game application. The plan focuses on testing the full application stack including web API endpoints, CLI interface, concurrent access patterns, and system behavior under various conditions.

## Goals & Objectives

### Primary Goals
- Establish robust integration testing for both CLI and web interfaces
- Ensure consistent behavior across different environments
- Test concurrent access and state management
- Validate API contracts and error handling
- Enable CI/CD pipeline integration

### Success Criteria
- 90%+ code coverage for integration paths
- All API endpoints tested with various scenarios
- Concurrent access patterns validated
- Tests run in isolated, reproducible environments
- Sub-5 minute test execution time

## Technology Stack

### Core Testing Framework
- **testcontainers-rs**: Rust implementation of test containers
- **bollard**: Docker API client for Rust
- **tokio-test**: Async test utilities
- **reqwest**: HTTP client for API testing
- **assert_cmd**: CLI testing framework

### Container Requirements
- **Application Container**: Containerized version of the game
- **Test Runner Container**: Isolated test execution environment
- **Network Container**: For testing network conditions
- **Load Testing Container**: For performance testing

## Implementation Phases

### Phase 1: Infrastructure Setup (Week 1)

#### 1.1 Project Configuration
```toml
# Cargo.toml additions
[dev-dependencies]
testcontainers = "0.15"
bollard = "0.15"
tokio-test = "0.4"
reqwest = { version = "0.12", features = ["json"] }
assert_cmd = "2.0"
predicates = "3.0"
serial_test = "3.0"
```

#### 1.2 Docker Configuration
```dockerfile
# Dockerfile for application
FROM rust:1.89-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/number_guessing_game /usr/local/bin/
EXPOSE 3000
ENTRYPOINT ["number_guessing_game"]
```

#### 1.3 Test Infrastructure Module
```rust
// tests/common/mod.rs
pub mod containers;
pub mod fixtures;
pub mod assertions;
```

### Phase 2: Web API Integration Tests (Week 2)

#### 2.1 Test Structure
```
tests/
├── integration/
│   ├── api/
│   │   ├── game_lifecycle_test.rs
│   │   ├── concurrent_games_test.rs
│   │   ├── error_handling_test.rs
│   │   └── performance_test.rs
│   ├── cli/
│   │   ├── game_flow_test.rs
│   │   └── input_validation_test.rs
│   └── system/
│       ├── memory_leak_test.rs
│       └── stress_test.rs
├── common/
│   ├── mod.rs
│   ├── containers.rs
│   ├── fixtures.rs
│   └── assertions.rs
└── fixtures/
    └── test_data.json
```

#### 2.2 API Test Scenarios

##### Game Lifecycle Tests
- Create new game with valid parameters
- Create game with edge case values (min=0, max=1000000)
- Make correct guess on first try
- Exhaust guess limit
- Handle game not found errors
- Validate game removal after completion

##### Concurrent Access Tests
- Create 100 simultaneous games
- Multiple clients guessing on same game
- Race condition testing for game state updates
- Memory usage under concurrent load

##### Error Handling Tests
- Invalid input ranges (min > max)
- Out of bounds guesses
- Malformed JSON requests
- Missing required fields
- Integer overflow attempts

### Phase 3: CLI Integration Tests (Week 3)

#### 3.1 CLI Test Implementation
```rust
// tests/integration/cli/game_flow_test.rs
use testcontainers::clients::Docker;
use assert_cmd::Command;

#[test]
fn test_complete_game_flow() {
    let docker = Docker::default();
    let container = GameContainer::new();
    let node = docker.run(container);
    
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "1", "--max", "10", "--limit", "5"]);
    // Test implementation
}
```

#### 3.2 CLI Test Scenarios
- Complete game with successful guess
- Game with exhausted attempts
- Input validation (negative numbers, non-numeric)
- Interrupt handling (Ctrl+C)
- Piped input testing

### Phase 4: System Integration Tests (Week 4)

#### 4.1 Performance Tests
```rust
// tests/integration/system/stress_test.rs
#[test]
fn test_concurrent_load() {
    // Spawn 1000 concurrent connections
    // Monitor memory usage
    // Validate response times < 100ms
    // Check for memory leaks
}
```

#### 4.2 Reliability Tests
- Server restart recovery
- Long-running stability (24-hour test)
- Resource exhaustion handling
- Network failure simulation

### Phase 5: CI/CD Integration (Week 5)

#### 5.1 GitHub Actions Workflow
```yaml
name: Integration Tests
on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    services:
      docker:
        image: docker:dind
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run Integration Tests
        run: |
          cargo test --test integration -- --test-threads=1
      - name: Upload Test Results
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: target/test-results/
```

#### 5.2 Test Reporting
- JUnit XML output for CI systems
- Code coverage reports (tarpaulin)
- Performance regression detection
- Test execution time tracking

## Test Container Configurations

### Application Container
```rust
// tests/common/containers.rs
use testcontainers::{core::WaitFor, Image};

pub struct GameServerContainer {
    port: u16,
}

impl Image for GameServerContainer {
    type Args = Vec<String>;

    fn name(&self) -> String {
        "number-guessing-game".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Server running"),
            WaitFor::http_status_on_port(self.port, "/health", 200),
        ]
    }
}
```

### Test Data Management
```rust
// tests/common/fixtures.rs
pub struct TestGame {
    pub min: i32,
    pub max: i32,
    pub limit: Option<u32>,
    pub expected_number: i32,
}

impl TestGame {
    pub fn default() -> Self {
        Self {
            min: 1,
            max: 100,
            limit: Some(10),
            expected_number: 50,
        }
    }
}
```

## Testing Patterns & Best Practices

### 1. Test Isolation
- Each test runs in its own container
- No shared state between tests
- Clean teardown after each test
- Use `serial_test` for resource-intensive tests

### 2. Deterministic Testing
- Seed random number generators
- Mock time-based functions
- Control concurrent execution order
- Predictable test data

### 3. Assertion Strategies
```rust
// Custom assertions for better error messages
pub trait GameAssertions {
    fn assert_game_created(&self);
    fn assert_guess_result(&self, expected: GuessResult);
    fn assert_game_ended(&self);
}
```

### 4. Performance Benchmarks
```rust
// Establish baseline metrics
const MAX_RESPONSE_TIME_MS: u64 = 100;
const MAX_MEMORY_MB: usize = 50;
const MAX_CONCURRENT_GAMES: usize = 10000;
```

## Resource Requirements

### Development Environment
- Docker Desktop or Docker Engine
- 8GB RAM minimum
- 10GB disk space for images
- Multi-core CPU for parallel tests

### CI/CD Environment
- Docker-in-Docker support
- 4GB RAM for test runners
- Artifact storage for test results
- Test result visualization tools

## Risk Mitigation

### Identified Risks
1. **Flaky Tests**: Mitigate with retry logic and proper wait conditions
2. **Resource Exhaustion**: Implement resource limits and cleanup
3. **Slow Test Execution**: Parallelize where possible, use test sharding
4. **Docker Compatibility**: Test on multiple Docker versions
5. **Network Issues**: Implement proper timeouts and retries

### Contingency Plans
- Fallback to unit tests if containers unavailable
- Local test mode without containers
- Gradual rollout with feature flags
- Rollback procedures for test infrastructure

## Success Metrics

### Quantitative Metrics
- Test execution time < 5 minutes
- Zero flaky tests over 100 runs
- 90%+ integration path coverage
- < 1% false positive rate
- All critical paths tested

### Qualitative Metrics
- Developer confidence in deployments
- Reduced production incidents
- Faster bug detection
- Improved code quality
- Better API documentation through tests

## Timeline & Milestones

### Week 1: Infrastructure Setup
- [ ] Add test container dependencies
- [ ] Create Dockerfile
- [ ] Setup test infrastructure module
- [ ] Write first smoke test

### Week 2: Web API Tests
- [ ] Implement game lifecycle tests
- [ ] Add concurrent access tests
- [ ] Create error handling tests
- [ ] Performance baseline tests

### Week 3: CLI Tests
- [ ] Game flow integration tests
- [ ] Input validation tests
- [ ] Error scenario tests
- [ ] CLI performance tests

### Week 4: System Tests
- [ ] Stress testing implementation
- [ ] Reliability tests
- [ ] Memory leak detection
- [ ] Long-running stability tests

## Maintenance & Evolution

### Regular Maintenance
- Weekly test suite health checks
- Monthly dependency updates
- Quarterly performance baseline review
- Annual architecture review

### Future Enhancements
1. **Contract Testing**: Add consumer-driven contracts
2. **Chaos Engineering**: Introduce failure injection
3. **Security Testing**: Add OWASP ZAP container
4. **Database Testing**: When persistence is added
5. **Multi-platform Testing**: Test on different OS containers

## Implementation Checklist

### Prerequisites
- [ ] Docker installed and configured
- [ ] Rust toolchain updated to 1.89+
- [ ] Team trained on test containers
- [ ] CI/CD permissions configured

### Development Tasks
- [ ] Create test directory structure
- [ ] Implement container configurations
- [ ] Write base test fixtures
- [ ] Create custom assertions
- [ ] Implement test scenarios
- [ ] Add performance benchmarks
- [ ] Configure CI/CD pipeline
- [ ] Update documentation

### Validation Steps
- [ ] All tests pass locally
- [ ] Tests run in CI/CD
- [ ] No flaky tests identified
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] Team sign-off obtained

## Conclusion

This plan provides a comprehensive approach to implementing integration testing using test containers. The phased approach ensures gradual adoption while maintaining system stability. By following this plan, the Number Guessing Game will have robust, reliable, and maintainable integration tests that provide confidence in deployments and catch issues early in the development cycle.

## Appendix A: Example Test Implementation

```rust
// Complete example of a game lifecycle test
use testcontainers::{clients::Docker, core::WaitFor};
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_complete_game_lifecycle() {
    // Setup
    let docker = Docker::default();
    let container = GameServerContainer::new(3000);
    let node = docker.run(container);
    let port = node.get_host_port_ipv4(3000);
    let client = Client::new();
    let base_url = format!("http://localhost:{}", port);
    
    // Create game
    let create_response = client
        .post(format!("{}/api/games", base_url))
        .json(&json!({
            "min": 1,
            "max": 100,
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(create_response.status(), 201);
    let game: Game = create_response.json().await.unwrap();
    
    // Make guess
    let guess_response = client
        .post(format!("{}/api/games/{}/guess", base_url, game.id))
        .json(&json!({ "guess": 50 }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(guess_response.status(), 200);
    
    // Verify game removed after completion
    let get_response = client
        .get(format!("{}/api/games/{}", base_url, game.id))
        .send()
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), 404);
}
```

## Appendix B: Resource Links

- [testcontainers-rs Documentation](https://github.com/testcontainers/testcontainers-rs)
- [Docker Best Practices for Testing](https://docs.docker.com/develop/dev-best-practices/)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [CI/CD Best Practices](https://www.atlassian.com/continuous-delivery/principles/continuous-integration-vs-delivery-vs-deployment)
- [Performance Testing Guidelines](https://www.guru99.com/performance-testing.html)