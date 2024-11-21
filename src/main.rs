use clap::Parser;
use clap_stdin::MaybeStdin;
use std::env;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
/// Only interprets `${}` formatted variables. Without brackets it will not be replaced.
/// NB: when providing the string via stdin, must pass '-' instead of normal arguments.
struct Cli {
	s: MaybeStdin<String>,
}

fn replace_env_vars(input: &str) -> String {
	let mut s = input.to_string();
	loop {
		let mut found = false;
		for i in 0..s.len() {
			if s[i..].starts_with("${") {
				let end = s[i + 2..].find('}').unwrap_or(s.len());
				let var = &s[i + 2..i + 2 + end];
				match env::var(var) {
					Ok(value) => {
						s = s.replacen(&format!("${{{}}}", var), &value, 1);
						found = true;
						break;
					}
					Err(e) => {
						eprintln!("Warning: {e}: {var}. Skipping.");
						// Skip over, without replacing
						s = s.replacen(&format!("${{{}}}", var), &format!("${{{}}}", var), 1);
					}
				}
			}
		}
		if !found {
			break;
		}
	}
	s
}

fn main() {
	let args = Cli::parse();
	let input = format!("{}", args.s);
	let output = replace_env_vars(&input);
	println!("{output}");
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	#[test]
	fn test_env_var_replacement() {
		env::set_var("KNOWN_VAR", "rust-lang");
		env::remove_var("VAR_THAT_DOES_NOT_EXIST");
		let input = "Hello, ${KNOWN_VAR}! Your flags are: ${VAR_THAT_DOES_NOT_EXIST}.";
		let expected_output = "Hello, rust-lang! Your flags are: ${VAR_THAT_DOES_NOT_EXIST}.";
		let output = replace_env_vars(input);
		assert_eq!(output, expected_output);
	}
}
