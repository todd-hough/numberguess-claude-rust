use std::io::{self, Write};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "number_guessing_game")]
#[command(about = "A fun number guessing game", long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Minimum number (inclusive)")]
    pub min: Option<i32>,
    
    #[arg(short = 'x', long, help = "Maximum number (inclusive)")]
    pub max: Option<i32>,
}

pub fn read_input<T>(prompt: &str) -> T 
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    loop {
        print!("{}", prompt);
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

pub fn get_valid_max(min: i32) -> i32 {
    loop {
        let max: i32 = read_input("Enter maximum number (inclusive): ");
        if max >= min {
            return max;
        } else {
            println!("Maximum must be greater than or equal to minimum. Please try again.");
        }
    }
}

pub fn get_min_value(cli_min: Option<i32>) -> i32 {
    match cli_min {
        Some(m) => {
            println!("Using minimum value from command line: {}", m);
            m
        },
        None => read_input("Enter minimum number (inclusive): ")
    }
}

pub fn get_max_value(cli_max: Option<i32>, min: i32) -> i32 {
    match cli_max {
        Some(m) => {
            if m >= min {
                println!("Using maximum value from command line: {}", m);
                m
            } else {
                println!("Maximum from command line ({}) is less than minimum ({}). Please provide a valid maximum.", m, min);
                get_valid_max(min)
            }
        },
        None => get_valid_max(min)
    }
}