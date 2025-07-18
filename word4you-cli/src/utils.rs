use anyhow::{anyhow, Result};

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn ensure_vocabulary_notebook_exists(vocabulary_notebook_file: &str) -> Result<()> {
    let path = Path::new(vocabulary_notebook_file);

    // Create word4you directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            println!("📁 Created word4you directory: {}", parent.display());
        }
    }

    // Create empty file if it doesn't exist
    if !path.exists() {
        File::create(vocabulary_notebook_file)?;
        println!(
            "📄 Created vocabulary notebook: {}",
            vocabulary_notebook_file
        );
    }
    Ok(())
}

pub fn prepend_to_vocabulary_notebook(vocabulary_notebook_file: &str, content: &str) -> Result<()> {
    ensure_vocabulary_notebook_exists(vocabulary_notebook_file)?;

    // Read existing content
    let existing_content = fs::read_to_string(vocabulary_notebook_file)?;

    // Generate local timestamp in ISO 8601 format with 3-digit milliseconds
    let local_timestamp = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    // Check if content already has timestamp and separator
    let formatted_content = if content.contains("<!-- timestamp=") && content.contains("---") {
        // Content is already formatted (e.g., from git sync), use as-is
        content.trim().to_string()
    } else {
        // Add timestamp and separator for new content
        format!(
            "{}\n\n<!-- timestamp={} -->\n\n---",
            content.trim(), local_timestamp
        )
    };

    // Prepend new content, ensuring proper spacing
    let new_content = if existing_content.trim().is_empty() {
        formatted_content
    } else {
        format!("{}\n{}", formatted_content, existing_content)
    };

    fs::write(vocabulary_notebook_file, new_content)?;

    Ok(())
}

pub fn delete_from_vocabulary_notebook(
    vocabulary_notebook_file: &str,
    word: &str,
    timestamp: Option<&str>,
) -> Result<()> {
    ensure_vocabulary_notebook_exists(vocabulary_notebook_file)?;

    // Open the file for reading
    let file = File::open(vocabulary_notebook_file)?;
    let reader = BufReader::new(file);

    let mut found = false;
    let lines: Vec<String> = reader.lines().collect::<std::result::Result<_, _>>()?;
    let mut filtered_content = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        println!("Processing line {}: {}", i + 1, line);

        // Check if this line starts a new word section
        if !found && line.starts_with("## ") {
            let section_word = line[3..].trim(); // Remove "## " prefix


            // Check if this is the section we want to delete
            if section_word.to_lowercase() == word.to_lowercase() {

                // If timestamp is specified, look for its exact match
                if let Some(ts) = timestamp {
                    // Search forward for the timestamp line within the current section
                    let mut j = i + 1;

                    // Look ahead to find the timestamp line before the section separator
                    while j < lines.len() && lines[j].trim() != "---" {
                        if lines[j].starts_with("<!-- timestamp=") && lines[j].contains(ts) {
                            found = true;
                            break;
                        }
                        j += 1;
                    }
                } else {
                    found = true;
                }
            }

            // Skip the entire section if it matches our criteria
            if found {
                // Look for the next separator or end of file
                while i < lines.len() && lines[i].trim() != "---" {
                    i += 1;
                }

                // Skip the separator line and the blank line after it
                if i < lines.len() && lines[i].trim() == "---" {
                    i += 1; // Skip the separator line
                }

                continue;
            }

        }

        // Add the line to our filtered content
        filtered_content.push(line.clone());

        i += 1;
    }

    if !found {
        return Err(anyhow!(
            "Word '{}' {} not found in vocabulary notebook",
            word,
            timestamp.map_or("".to_string(), |ts| format!("with timestamp {}", ts))
        ));
    }

    // Only write to file if we found the word
    fs::write(vocabulary_notebook_file, filtered_content.join("\n"))?;

    Ok(())
}

pub fn validate_word(word: &str) -> Result<()> {
    if word.trim().is_empty() {
        return Err(anyhow!("Word cannot be empty"));
    }

    let word = word.trim();

    // Check if it contains only letters and hyphens
    if !word.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
        return Err(anyhow!("Word can only contain letters and hyphens"));
    }

    // Check length
    if word.len() < 1 || word.len() > 50 {
        return Err(anyhow!("Word length must be between 1 and 50 characters"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_prepend_to_vocabulary_notebook() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab.md");
        let temp_file = file_path.to_str().unwrap();

        prepend_to_vocabulary_notebook(temp_file, "Test word content").unwrap();

        let result = fs::read_to_string(temp_file).unwrap();

        let timestamp_regex = Regex::new(
            r"<!-- timestamp=\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}[+-]\d{2}:\d{2} -->",
        )
        .unwrap();
        assert!(timestamp_regex.is_match(&result));
        assert!(result.contains("Test word content"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_validate_word() {
        assert!(validate_word("hello").is_ok());
        assert!(validate_word("test-word").is_ok());
        assert!(validate_word("a").is_ok());

        assert!(validate_word("").is_err());
        assert!(validate_word("   ").is_err());
        assert!(validate_word("hello123").is_err());
        assert!(validate_word("hello@world").is_err());

        let long_word = "a".repeat(51);
        assert!(validate_word(&long_word).is_err());
    }

    #[test]
    fn test_delete_with_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(temp_file, "## hello\nHello content\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n\n---\n## world\nWorld content\n\n<!-- timestamp=2023-01-02T12:00:00.456+00:00 -->\n\n---").unwrap();

        delete_from_vocabulary_notebook(temp_file, "hello", Some("2023-01-01T12:00:00.123+00:00"))
            .unwrap();

        let result = fs::read_to_string(temp_file).unwrap();
        println!("Result after deletion:\n{}", result);

        assert!(result.contains("## world"));
        assert!(!result.contains("## hello"));
    }

    #[test]
    fn test_delete_without_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete_all.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(temp_file, "## hello\nHello content 1\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n---\n\n## word\nHello content 2\n\n<!-- timestamp=2023-01-02T12:00:00.456+00:00 -->\n---\n").unwrap();

        delete_from_vocabulary_notebook(temp_file, "hello", None).unwrap();

        let result = fs::read_to_string(temp_file).unwrap();

        assert!(!result.contains("## hello"));
    }

    #[test]
    fn test_delete_nonexistent_word_with_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete_nonexistent.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(
            temp_file,
            "## hello\nHello content\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n---\n",
        )
        .unwrap();

        let result = delete_from_vocabulary_notebook(
            temp_file,
            "world",
            Some("2023-01-01T12:00:00.123+00:00"),
        );

        assert!(result.is_err());

        let result_content = fs::read_to_string(temp_file).unwrap();
        assert!(result_content.contains("## hello"));
    }
}
