use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_hash_calculation() {
    // Create a temporary test file with exact content
    let test_content = "test_content";
    let test_file_path = create_test_file(test_content).expect("Failed to create test file");
    
    // Calculate the expected hash for verification
    let expected_hash = calculate_expected_hash(test_file_path.to_str().unwrap());
    
    // Get the hash using the CLI mode
    let output = Command::new("target/debug/hashsafe")
        .args(["--file", test_file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Extract the actual hash from the output
    let hash_line = output_str.lines()
        .find(|line| line.starts_with("SHA-256 Hash:"))
        .expect("Hash line not found in output");
    
    let actual_hash = hash_line.trim_start_matches("SHA-256 Hash:").trim();
    
    // Check if the output contains the expected hash
    assert_eq!(expected_hash, actual_hash, 
        "Hash mismatch.\nExpected hash: {}\nActual hash: {}", 
        expected_hash, actual_hash);
    
    // Clean up the test file
    std::fs::remove_file(test_file_path).expect("Failed to remove test file");
}

// Helper function to create a temporary test file with specified content
fn create_test_file(content: &str) -> std::io::Result<PathBuf> {
    let test_dir = std::env::temp_dir();
    let file_path = test_dir.join("hashsafe_test_file.txt");
    
    // Make sure we create a new file or truncate existing one
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    
    Ok(file_path)
}

// Calculate the expected hash using the system's sha256sum tool
fn calculate_expected_hash(file_path: &str) -> String {
    // On macOS, we'll use shasum -a 256
    let output = Command::new("shasum")
        .args(["-a", "256", file_path])
        .output()
        .expect("Failed to execute shasum command");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Extract just the hash part (first 64 characters)
    output_str.split_whitespace().next().unwrap_or("").to_string()
}
