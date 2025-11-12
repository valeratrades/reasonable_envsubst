use clap::Parser;
use clap_stdin::MaybeStdin;
use std::env;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
/// Interprets environment variables in two formats:
/// - `${}` pattern: ${VARIABLE_NAME} -> expanded value
/// - Object pattern: { env = "VARIABLE_NAME" } -> "expanded value"
/// Without these patterns, text will not be replaced.
/// NB: when providing the string via stdin, must pass '-' instead of normal arguments.
struct Cli {
	s: MaybeStdin<String>,
}

fn replace_env_vars(input: &str) -> String {
	let mut s = input.to_string();
	loop {
		let mut found = false;
		for i in 0..s.len() {
			// Check for ${ pattern
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
			// Check for { env = "VARIABLE_NAME" } pattern
			else if s[i..].starts_with("{ env = \"") {
				if let Some(quote_end) = s[i + 9..].find('"') {
					if let Some(brace_end) = s[i + 9 + quote_end..].find(" }") {
						let var = &s[i + 9..i + 9 + quote_end];
						let pattern_end = i + 9 + quote_end + brace_end + 2;
						let full_pattern = &s[i..pattern_end];

						match env::var(var) {
							Ok(value) => {
								// Replace the entire pattern with just the quoted value
								s = s.replacen(full_pattern, &format!("\"{}\"", value), 1);
								found = true;
								break;
							}
							Err(e) => {
								eprintln!("Warning: {e}: {var}. Skipping.");
								// Skip over this pattern without replacing
								// We need to mark this as processed to avoid infinite loop
								s = s.replacen(full_pattern, full_pattern, 1);
							}
						}
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

	#[test]
	fn test_env_object_pattern_replacement() {
		env::set_var("MY_VAR", "my-value");
		env::set_var("ANOTHER_VAR", "another-value");
		env::remove_var("UNKNOWN_VAR");

		let input = r#"Config: { env = "MY_VAR" } and { env = "ANOTHER_VAR" }, unknown: { env = "UNKNOWN_VAR" }"#;
		let expected_output = r#"Config: "my-value" and "another-value", unknown: { env = "UNKNOWN_VAR" }"#;
		let output = replace_env_vars(input);
		assert_eq!(output, expected_output);
	}

	#[test]
	fn test_mixed_patterns() {
		env::set_var("VAR1", "value1");
		env::set_var("VAR2", "value2");

		let input = r#"Old style: ${VAR1}, new style: { env = "VAR2" }"#;
		let expected_output = r#"Old style: value1, new style: "value2""#;
		let output = replace_env_vars(input);
		assert_eq!(output, expected_output);
	}
}
