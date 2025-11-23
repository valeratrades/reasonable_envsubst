use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

fn get_binary_path() -> Option<String> {
	let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
	let debug_path = format!("{}/target/debug/reasonable_envsubst", cargo_manifest_dir);
	let release_path = format!("{}/target/release/reasonable_envsubst", cargo_manifest_dir);

	if Path::new(&debug_path).exists() {
		Some(debug_path)
	} else if Path::new(&release_path).exists() {
		Some(release_path)
	} else {
		None
	}
}

#[test]
fn test_dollar_brace_pattern() {
	let Some(binary_path) = get_binary_path() else {
		eprintln!("Skipping integration test: binary not found in target directory");
		return;
	};
	env::set_var("KNOWN_VAR", "rust-lang");
	env::set_var("VAR1", "value1");
	env::set_var("MULTIPLE1", "first");
	env::set_var("MULTIPLE2", "second");
	env::remove_var("UNKNOWN_VAR");

	// Test single replacement
	let input = "Hello, ${KNOWN_VAR}!";
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Hello, rust-lang!");

	// Test unknown variable (should skip)
	let input = "Known: ${KNOWN_VAR}, unknown: ${UNKNOWN_VAR}";
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Known: rust-lang, unknown: ${UNKNOWN_VAR}");

	// Test multiple replacements
	let input = "${MULTIPLE1} and ${MULTIPLE2}";
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "first and second");

	// Test from stdin
	let input = "From stdin: ${VAR1}";
	let mut temp_file = NamedTempFile::new().unwrap();
	temp_file.write_all(input.as_bytes()).unwrap();
	temp_file.flush().unwrap();

	let output = Command::new(&binary_path)
		.arg("-")
		.stdin(std::fs::File::open(temp_file.path()).unwrap())
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "From stdin: value1");
}

#[test]
fn test_env_object_pattern() {
	let Some(binary_path) = get_binary_path() else {
		eprintln!("Skipping integration test: binary not found in target directory");
		return;
	};

	env::set_var("MY_VAR", "my-value");
	env::set_var("VAR2", "value2");
	env::set_var("A", "apple");
	env::set_var("B", "banana");
	env::remove_var("UNKNOWN_VAR");

	// Test single replacement
	let input = r#"Config: { env = "MY_VAR" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"Config: "my-value""#);

	// Test unknown variable (should skip)
	let input = r#"Known: { env = "MY_VAR" }, unknown: { env = "UNKNOWN_VAR" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"Known: "my-value", unknown: { env = "UNKNOWN_VAR" }"#);

	// Test multiple replacements
	let input = r#"{ env = "A" } and { env = "B" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#""apple" and "banana""#);

	// Test from stdin with mixed patterns
	let input = r#"From stdin: { env = "VAR2" } and ${VAR2}"#;
	let mut temp_file = NamedTempFile::new().unwrap();
	temp_file.write_all(input.as_bytes()).unwrap();
	temp_file.flush().unwrap();

	let output = Command::new(&binary_path)
		.arg("-")
		.stdin(std::fs::File::open(temp_file.path()).unwrap())
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"From stdin: "value2" and value2"#);
}

#[test]
fn test_commented_lines_ignored() {
	let Some(binary_path) = get_binary_path() else {
		eprintln!("Skipping integration test: binary not found in target directory");
		return;
	};

	env::set_var("COMMENT_VAR", "should-not-replace");
	env::set_var("ACTIVE_VAR", "should-replace");

	// Test single line comment with { env = "..." } pattern
	let input = r#"# { env = "COMMENT_VAR" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"# { env = "COMMENT_VAR" }"#);

	// Test comment with leading spaces
	let input = r#"   # { env = "COMMENT_VAR" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), r#"   # { env = "COMMENT_VAR" }"#);

	// Test multiline with mix of commented and uncommented lines
	let input = r#"active: { env = "ACTIVE_VAR" }
# commented: { env = "COMMENT_VAR" }
another: { env = "ACTIVE_VAR" }"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"active: "should-replace"
# commented: { env = "COMMENT_VAR" }
another: "should-replace""#);

	// Test from stdin with commented lines
	let input = r#"# Config comment: { env = "COMMENT_VAR" }
real_value: { env = "ACTIVE_VAR" }
  # Another comment: { env = "COMMENT_VAR" }"#;
	let mut temp_file = NamedTempFile::new().unwrap();
	temp_file.write_all(input.as_bytes()).unwrap();
	temp_file.flush().unwrap();

	let output = Command::new(&binary_path)
		.arg("-")
		.stdin(std::fs::File::open(temp_file.path()).unwrap())
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"# Config comment: { env = "COMMENT_VAR" }
real_value: "should-replace"
  # Another comment: { env = "COMMENT_VAR" }"#);

	// Verify ${} pattern still works in comments (not affected by comment logic)
	let input = r#"# This is a comment ${COMMENT_VAR}"#;
	let output = Command::new(&binary_path)
		.arg(input)
		.output()
		.expect("Failed to execute binary");
	assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#"# This is a comment should-not-replace"#);
}
