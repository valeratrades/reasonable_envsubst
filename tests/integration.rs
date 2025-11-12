use std::env;
use std::fs;
use std::process::Command;

fn get_binary_path() -> String {
	let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
	format!("{}/target/debug/reasonable_envsubst", cargo_manifest_dir)
}

#[test]
fn test_dollar_brace_pattern() {
	env::set_var("KNOWN_VAR", "rust-lang");
	env::set_var("VAR1", "value1");
	env::set_var("MULTIPLE1", "first");
	env::set_var("MULTIPLE2", "second");
	env::remove_var("UNKNOWN_VAR");

	let temp_file = "/tmp/test_dollar_brace.txt";

	// Test single replacement
	let input = "Hello, ${KNOWN_VAR}!";
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Hello, rust-lang!");

	// Test unknown variable (should skip)
	let input = "Known: ${KNOWN_VAR}, unknown: ${UNKNOWN_VAR}";
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Known: rust-lang, unknown: ${UNKNOWN_VAR}");

	// Test multiple replacements
	let input = "${MULTIPLE1} and ${MULTIPLE2}";
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "first and second");

	// Test from stdin
	let input = "From stdin: ${VAR1}";
	fs::write(temp_file, input).unwrap();
	let output = Command::new(get_binary_path())
		.arg("-")
		.stdin(fs::File::open(temp_file).unwrap())
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "From stdin: value1");

	fs::remove_file(temp_file).ok();
}

#[test]
fn test_env_object_pattern() {
	env::set_var("MY_VAR", "my-value");
	env::set_var("VAR2", "value2");
	env::set_var("A", "apple");
	env::set_var("B", "banana");
	env::remove_var("UNKNOWN_VAR");

	let temp_file = "/tmp/test_env_object.txt";

	// Test single replacement
	let input = r#"Config: { env = "MY_VAR" }"#;
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"Config: "my-value""#);

	// Test unknown variable (should skip)
	let input = r#"Known: { env = "MY_VAR" }, unknown: { env = "UNKNOWN_VAR" }"#;
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"Known: "my-value", unknown: { env = "UNKNOWN_VAR" }"#);

	// Test multiple replacements
	let input = r#"{ env = "A" } and { env = "B" }"#;
	let output = Command::new(get_binary_path())
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#""apple" and "banana""#);

	// Test from stdin with mixed patterns
	let input = r#"From stdin: { env = "VAR2" } and ${VAR2}"#;
	fs::write(temp_file, input).unwrap();
	let output = Command::new(get_binary_path())
		.arg("-")
		.stdin(fs::File::open(temp_file).unwrap())
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"From stdin: "value2" and value2"#);

	fs::remove_file(temp_file).ok();
}
