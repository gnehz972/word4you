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
            content.trim(),
            local_timestamp
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
    timestamp: &str,
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

        // Check if this line starts a new word section
        if !found && line.starts_with("## ") {
            // Search forward for the timestamp line within the current section
            let mut j = i + 1;

            // Look ahead to find the timestamp line before the section separator
            while j < lines.len() && lines[j].trim() != "---" {
                if lines[j].starts_with("<!-- timestamp=") && lines[j].contains(timestamp) {
                    found = true;
                    break;
                }
                j += 1;
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
            "Entry with timestamp '{}' not found in vocabulary notebook",
            timestamp
        ));
    }

    // Only write to file if we found the word
    fs::write(vocabulary_notebook_file, filtered_content.join("\n"))?;

    Ok(())
}

pub fn is_chinese_ideograph(c: char) -> bool {
    // Chinese characters are in the CJK Unified Ideographs block (U+4E00 to U+9FFF)
    // and some other CJK blocks
    (c >= '\u{4E00}' && c <= '\u{9FFF}') || // Basic CJK Unified Ideographs
    (c >= '\u{3400}' && c <= '\u{4DBF}') || // CJK Unified Ideographs Extension A
    (c >= '\u{20000}' && c <= '\u{2A6DF}') || // CJK Unified Ideographs Extension B
    (c >= '\u{2A700}' && c <= '\u{2B73F}') || // CJK Unified Ideographs Extension C
    (c >= '\u{2B740}' && c <= '\u{2B81F}') || // CJK Unified Ideographs Extension D
    (c >= '\u{2B820}' && c <= '\u{2CEAF}') || // CJK Unified Ideographs Extension E
    (c >= '\u{2CEB0}' && c <= '\u{2EBEF}') || // CJK Unified Ideographs Extension F
    (c >= '\u{30000}' && c <= '\u{3134F}') // CJK Unified Ideographs Extension G
}

fn is_chinese_punctuation(c: char) -> bool {
    // Check for common Chinese punctuation marks
    // This is not an exhaustive list, but covers many frequently used ones.
    matches!(
        c,
        // Full-width forms (often used in Chinese)
        '\u{3001}' | // IDEOGRAPHIC COMMA (、)
        '\u{3002}' | // IDEOGRAPHIC FULL STOP (。)
        '\u{FF0C}' | // FULLWIDTH COMMA (，)
        '\u{FF0E}' | // FULLWIDTH FULL STOP (．)
        '\u{FF1B}' | // FULLWIDTH SEMICOLON (；)
        '\u{FF1A}' | // FULLWIDTH COLON (：)
        '\u{FF1F}' | // FULLWIDTH QUESTION MARK (？)
        '\u{FF01}' | // FULLWIDTH EXCLAMATION MARK (！)
        '\u{3010}' | // LEFT BLACK LENTICULAR BRACKET (【)
        '\u{3011}' | // RIGHT BLACK LENTICULAR BRACKET (】)
        '\u{FF08}' | // FULLWIDTH LEFT PARENTHESIS (（)
        '\u{FF09}' | // FULLWIDTH RIGHT PARENTHESIS (）)
        '\u{300A}' | // LEFT DOUBLE ANGLE BRACKET (《)
        '\u{300B}' | // RIGHT DOUBLE ANGLE BRACKET (》)
        '\u{3008}' | // LEFT ANGLE BRACKET (〈)
        '\u{3009}' | // RIGHT ANGLE BRACKET (〉)
        '\u{2014}' | // EM DASH (—) - Often used in Chinese
        '\u{2018}' | // LEFT SINGLE QUOTATION MARK (‘)
        '\u{2019}' | // RIGHT SINGLE QUOTATION MARK (’)
        '\u{201C}' | // LEFT DOUBLE QUOTATION MARK (“)
        '\u{201D}' // RIGHT DOUBLE QUOTATION MARK (”)
                   // Add more as needed based on requirements
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    English,
    Chinese,
    Mixed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Word,
    Phrase,
    Sentence,
}

#[derive(Debug, Clone)]
pub struct InputClassification {
    pub language: Language,
    pub input_type: InputType,
}

pub fn classify_input(input: &str) -> InputClassification {
    let input = input.trim();

    // Determine language
    let language = determine_language(input);

    // Determine input type
    let input_type = determine_input_type(input, &language);

    InputClassification {
        language,
        input_type,
    }
}

fn determine_language(input: &str) -> Language {
    let total_chars = input.chars().count();
    if total_chars == 0 {
        return Language::English; // Default fallback
    }

    let chinese_char_count = input.chars().filter(|c| is_chinese_ideograph(*c)).count();
    let chinese_punct_count = input.chars().filter(|c| is_chinese_punctuation(*c)).count();
    let chinese_total = chinese_char_count + chinese_punct_count;

    // Count non-whitespace characters for better ratio calculation
    let non_whitespace_chars = input.chars().filter(|c| !c.is_whitespace()).count();

    if non_whitespace_chars == 0 {
        return Language::English;
    }

    let chinese_ratio = chinese_total as f64 / non_whitespace_chars as f64;

    if chinese_ratio >= 0.6 {
        Language::Chinese
    } else if chinese_ratio > 0.0 && chinese_total > 0 {
        // If there are any Chinese characters, it's mixed
        Language::Mixed
    } else {
        Language::English
    }
}

fn determine_input_type(input: &str, language: &Language) -> InputType {
    let input = input.trim();

    // Count spaces and words
    let space_count = input.chars().filter(|c| c.is_whitespace()).count();
    let word_count = if space_count == 0 { 1 } else { space_count + 1 };

    // Check for sentence-ending punctuation
    let has_sentence_ending = input
        .chars()
        .any(|c| matches!(c, '.' | '!' | '?' | '。' | '！' | '？' | '…' | '：' | ':'));

    // Count Chinese characters
    let chinese_char_count = input.chars().filter(|c| is_chinese_ideograph(*c)).count();

    match language {
        Language::Chinese | Language::Mixed => {
            if chinese_char_count == 1 && space_count == 0 {
                // Single Chinese character
                InputType::Word
            } else if has_sentence_ending || chinese_char_count >= 8 {
                // Has sentence punctuation or many Chinese characters
                InputType::Sentence
            } else if chinese_char_count >= 2 && chinese_char_count <= 7 {
                // 2-7 Chinese characters, likely a phrase
                InputType::Phrase
            } else {
                // Fallback based on word count
                if word_count == 1 {
                    InputType::Word
                } else if word_count <= 4 {
                    InputType::Phrase
                } else {
                    InputType::Sentence
                }
            }
        }
        Language::English => {
            if word_count == 1 && !has_sentence_ending {
                // Single English word
                InputType::Word
            } else if has_sentence_ending || word_count >= 6 {
                // Has sentence punctuation or many words
                InputType::Sentence
            } else {
                // 2-5 words, likely a phrase
                InputType::Phrase
            }
        }
    }
}

pub fn validate_text(text: &str) -> Result<()> {
    if text.trim().is_empty() {
        return Err(anyhow!("Input cannot be empty"));
    }

    let text = text.trim();

    // Allow letters (including CJK), digits, punctuation, and spaces for phrases and sentences
    if !text.chars().all(|c| {
        c.is_alphabetic()
            || c.is_ascii_digit()
            || c.is_ascii_punctuation()
            || c.is_ascii_whitespace()
            || is_chinese_ideograph(c)
            || is_chinese_punctuation(c)
    }) {
        return Err(anyhow!(
            "Input can only contain letters, numbers, punctuation, and spaces"
        ));
    }

    // Ensure the input contains at least one letter (alphabetic character)
    if !text
        .chars()
        .any(|c| c.is_alphabetic() || is_chinese_ideograph(c))
    {
        return Err(anyhow!("Input must contain at least one letter"));
    }

    // Check length
    if text.len() < 1 || text.len() > 200 {
        return Err(anyhow!("Input length must be between 1 and 200 characters"));
    }

    Ok(())
}

pub fn get_work_dir(vocabulary_notebook_file: &str) -> Result<&Path> {
    let notebook_path = Path::new(vocabulary_notebook_file);
    let work_dir = notebook_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary notebook file path"))?;
    Ok(work_dir)
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
    fn test_validate_text() {
        assert!(validate_text("hello").is_ok());
        assert!(validate_text("test-word").is_ok());
        assert!(validate_text("a").is_ok());
        assert!(validate_text("你好").is_ok());
        assert!(validate_text("Hello world").is_ok());
        assert!(validate_text("这是一个句子。").is_ok());

        assert!(validate_text("").is_err());
        assert!(validate_text("   ").is_err());

        let long_text = "a".repeat(201);
        assert!(validate_text(&long_text).is_err());
    }

    #[test]
    fn test_language_classification() {
        // English
        assert_eq!(determine_language("hello"), Language::English);
        assert_eq!(determine_language("Hello world!"), Language::English);

        // Chinese
        assert_eq!(determine_language("你好"), Language::Chinese);
        assert_eq!(determine_language("这是一个测试。"), Language::Chinese);

        // Mixed
        assert_eq!(determine_language("Hello 你好"), Language::Mixed);
        assert_eq!(determine_language("API接口"), Language::Mixed);
    }

    #[test]
    fn test_input_type_classification() {
        // English words
        let classification = classify_input("hello");
        assert_eq!(classification.language, Language::English);
        assert_eq!(classification.input_type, InputType::Word);

        // English phrases
        let classification = classify_input("break the ice");
        assert_eq!(classification.language, Language::English);
        assert_eq!(classification.input_type, InputType::Phrase);

        // English sentences
        let classification = classify_input("The early bird catches the worm.");
        assert_eq!(classification.language, Language::English);
        assert_eq!(classification.input_type, InputType::Sentence);

        // Chinese words
        let classification = classify_input("你好");
        assert_eq!(classification.language, Language::Chinese);
        assert_eq!(classification.input_type, InputType::Phrase); // 2 characters = phrase

        let classification = classify_input("好");
        assert_eq!(classification.language, Language::Chinese);
        assert_eq!(classification.input_type, InputType::Word);

        // Chinese phrases
        let classification = classify_input("打破僵局");
        assert_eq!(classification.language, Language::Chinese);
        assert_eq!(classification.input_type, InputType::Phrase);

        // Chinese sentences
        let classification = classify_input("早起的鸟儿有虫吃。");
        assert_eq!(classification.language, Language::Chinese);
        assert_eq!(classification.input_type, InputType::Sentence);

        // Mixed language
        let classification = classify_input("Hello 世界");
        assert_eq!(classification.language, Language::Mixed);
    }

    #[test]
    fn test_chinese_character_detection() {
        assert!(is_chinese_ideograph('你'));
        assert!(is_chinese_ideograph('好'));
        assert!(is_chinese_ideograph('世'));
        assert!(is_chinese_ideograph('界'));

        assert!(!is_chinese_ideograph('a'));
        assert!(!is_chinese_ideograph('A'));
        assert!(!is_chinese_ideograph('1'));
        assert!(!is_chinese_ideograph(' '));
    }

    #[test]
    fn test_chinese_punctuation_detection() {
        assert!(is_chinese_punctuation('。'));
        assert!(is_chinese_punctuation('，'));
        assert!(is_chinese_punctuation('？'));
        assert!(is_chinese_punctuation('！'));

        assert!(!is_chinese_punctuation('.'));
        assert!(!is_chinese_punctuation(','));
        assert!(!is_chinese_punctuation('?'));
        assert!(!is_chinese_punctuation('!'));
    }

    #[test]
    fn test_delete_with_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(temp_file, "## hello\nHello content\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n\n---\n## world\nWorld content\n\n<!-- timestamp=2023-01-02T12:00:00.456+00:00 -->\n\n---").unwrap();

        delete_from_vocabulary_notebook(temp_file, "2023-01-01T12:00:00.123+00:00").unwrap();

        let result = fs::read_to_string(temp_file).unwrap();
        println!("Result after deletion:\n{}", result);

        assert!(result.contains("## world"));
        assert!(!result.contains("## hello"));
    }

    #[test]
    fn test_delete_by_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete_all.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(temp_file, "## hello\nHello content 1\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n---\n\n## word\nHello content 2\n\n<!-- timestamp=2023-01-02T12:00:00.456+00:00 -->\n---\n").unwrap();

        delete_from_vocabulary_notebook(temp_file, "2023-01-01T12:00:00.123+00:00").unwrap();

        let result = fs::read_to_string(temp_file).unwrap();

        assert!(!result.contains("## hello"));
        assert!(result.contains("## word"));
    }

    #[test]
    fn test_delete_nonexistent_timestamp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_vocab_delete_nonexistent.md");
        let temp_file = file_path.to_str().unwrap();

        fs::write(
            temp_file,
            "## hello\nHello content\n\n<!-- timestamp=2023-01-01T12:00:00.123+00:00 -->\n---\n",
        )
        .unwrap();

        let result = delete_from_vocabulary_notebook(temp_file, "2023-01-01T12:00:00.999+00:00");

        assert!(result.is_err());

        let result_content = fs::read_to_string(temp_file).unwrap();
        assert!(result_content.contains("## hello"));
    }
}
