use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_basic_game_flow() {
    // Test with a fixed number range where min=max to guarantee the answer
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "5", "--max", "5", "--limit", "1"])
        .write_stdin("5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Welcome to the Number Guessing Game!",
        ))
        .stdout(predicate::str::contains("You got it! The number was 5"));
}

#[test]
fn test_cli_with_wrong_guess() {
    // Test with wrong guess and limit reached
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "5", "--max", "5", "--limit", "1"])
        .write_stdin("3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("you've reached the limit"))
        .stdout(predicate::str::contains("The number was 5"));
}

#[test]
fn test_cli_help_output() {
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A fun number guessing game"))
        .stdout(predicate::str::contains("--min"))
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_cli_server_mode() {
    // Test that server mode can be invoked (we'll kill it immediately)
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--server", "--port", "0"]) // Port 0 lets OS assign
        .timeout(std::time::Duration::from_millis(100))
        .assert()
        .failure(); // Will fail due to timeout, which is expected
}

#[test]
fn test_cli_multiple_guesses() {
    // Test multiple guesses with a small range
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "1", "--max", "3", "--limit", "3"])
        .write_stdin("1\n2\n3\n") // Try all possibilities
        .assert()
        .success()
        .stdout(predicate::str::contains("You got it!"));
}

#[test]
fn test_cli_invalid_input_recovery() {
    // Test that invalid input is handled gracefully
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "5", "--max", "5"])
        .write_stdin("n\nabc\n5\n") // n for no limit, then invalid guess, then valid guess
        .assert()
        .success()
        .stdout(predicate::str::contains("Invalid input. Please try again."))
        .stdout(predicate::str::contains("You got it!"));
}
