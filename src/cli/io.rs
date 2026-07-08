//! User input/output helpers for CLI interactions.
//!
//! This module contains I/O functions for reading user input and displaying prompts,
//! with no validation logic (validation is in the validators module).

use std::io::{self, Write};

use crate::core::validators;

/// Reads a value from stdin with a prompt, retrying until valid input is received
pub fn read_input<T>(prompt: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    loop {
        print!("{prompt}");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

/// Prompts for and validates a minimum value
pub fn prompt_min_value(cli_min: Option<i32>) -> i32 {
    if let Some(m) = cli_min {
        match validators::validate_min_value(m) {
            Ok(()) => {
                println!("Using minimum value from command line: {m}");
                return m;
            }
            Err(e) => println!("{e}. Please provide a valid minimum."),
        }
    }
    prompt_valid_min()
}

/// Prompts for a valid minimum value (helper function, mirrors prompt_valid_max)
fn prompt_valid_min() -> i32 {
    loop {
        let min: i32 = read_input(&format!(
            "Enter minimum number (inclusive, 0 to {}): ",
            validators::MAX_RANGE
        ));
        if let Err(e) = validators::validate_min_value(min) {
            println!("{e}. Please try again.");
            continue;
        }
        return min;
    }
}

/// Prompts for and validates a maximum value
pub fn prompt_max_value(cli_max: Option<i32>, min: i32) -> i32 {
    match cli_max {
        Some(m) => {
            // Validate the max value itself
            if let Err(e) = validators::validate_max_value(m) {
                println!("{e}. Please provide a valid maximum.");
                return prompt_valid_max(min);
            }

            // Validate max >= min
            if let Err(e) = validators::validate_max_gte_min(min, m) {
                println!("{e}. Please provide a valid maximum.");
                return prompt_valid_max(min);
            }

            println!("Using maximum value from command line: {m}");
            m
        }
        None => prompt_valid_max(min),
    }
}

/// Prompts for a valid maximum value (helper function)
fn prompt_valid_max(min: i32) -> i32 {
    loop {
        let max: i32 = read_input(&format!(
            "Enter maximum number (inclusive, 0 to {}): ",
            validators::MAX_RANGE
        ));

        if let Err(e) = validators::validate_max_value(max) {
            println!("{e}. Please try again.");
            continue;
        }

        if let Err(e) = validators::validate_max_gte_min(min, max) {
            println!("{e}. Please try again.");
            continue;
        }

        return max;
    }
}

/// Prompts for and validates a guess limit
pub fn prompt_guess_limit(cli_limit: Option<u32>) -> Option<u32> {
    match cli_limit {
        Some(limit) => {
            match validators::validate_guess_limit(limit, validators::MAX_CLI_GUESS_LIMIT) {
                Ok(None) => {
                    println!("Playing without a guess limit.");
                    None
                }
                Ok(Some(validated_limit)) => {
                    if validated_limit < limit {
                        println!(
                            "Guess limit ({}) is very high. Using a maximum limit of {}.",
                            limit,
                            validators::MAX_CLI_GUESS_LIMIT
                        );
                    } else {
                        println!("Using guess limit from command line: {limit} guesses");
                    }
                    Some(validated_limit)
                }
                Err(e) => {
                    println!("{e}. Playing without a limit.");
                    None
                }
            }
        }
        None => {
            // Ask the user if they want to set a guess limit
            print!("Would you like to set a maximum number of guesses? (y/n): ");
            io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if input.trim().to_lowercase() == "y" {
                loop {
                    let limit: u32 = read_input(&format!(
                        "Enter maximum number of guesses (1-{}, or 0 for no limit): ",
                        validators::MAX_CLI_GUESS_LIMIT
                    ));
                    match validators::validate_guess_limit(limit, validators::MAX_CLI_GUESS_LIMIT) {
                        Ok(None) => {
                            println!("Playing without a guess limit.");
                            return None;
                        }
                        Ok(Some(validated_limit)) => {
                            println!("Guess limit set to {validated_limit} guesses.");
                            return Some(validated_limit);
                        }
                        Err(e) => {
                            println!("{e}. Please try again.");
                        }
                    }
                }
            } else {
                println!("Playing without a guess limit.");
                None
            }
        }
    }
}
