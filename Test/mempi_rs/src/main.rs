// run with cargo run <file with pi> <guess for pi>

use std::cmp;
use std::env;
use std::fs;
use std::process;

fn compare(real_pi: String, guessed_pi: String) {
    let end_pos = cmp::min(real_pi.len(), guessed_pi.len());
    let mut pos = 0;
    while pos < end_pos {
        if real_pi.chars().nth(pos) != guessed_pi.chars().nth(pos) {
            println!("Wrong on digit {}. You typed {} but it should have been \
                     {}.", pos, guessed_pi.chars().nth(pos).unwrap(),
                     real_pi.chars().nth(pos).unwrap());
            break;
        }
        pos = pos + 1;
    }
    if pos == end_pos {
        let next_digits : String = real_pi.chars().skip(pos).take(5).collect();
        println!("Correct for {} digits. Next {} digits are {}.", pos-1,
                 next_digits.len(), next_digits);
    }
}

fn main() {
    if env::args().len() != 3 {
        eprintln!("Error: Should have exactly two arguments (file with pi, \
                   guess for pi).");
      process::exit(1);
    }
    let pi_file = fs::read_to_string(env::args().nth(1).unwrap());
    if pi_file.is_err() {
        eprintln!("Error: could not open pi text file.");
        process::exit(1);
    }
    let real_pi = pi_file.unwrap();
    compare(real_pi, env::args().nth(2).unwrap());
}
