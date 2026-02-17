//! > **A lightweight utility for generating and managing environment variable files (.env) from the command line.**
//!
//! ## Install
//! ```console
//! $ cargo install env-gen
//! ```
//!
//! ## Usage
//! ```console
//! $ env-gen
//! ```
//! This will prompt you to create a new configuration file, optionally using an existing template file, and add key-value pairs for your environment variables. The generated `.env` file will be created in the current working directory.
//!
//! The following template files are supported (searched in the current working directory):
//! - `example.env`
//! - `.example.env`
//! - `env.example`
//! - `.env.example`
//! - `env.sample`
//! - `.env.sample`
//! - `.env-dist`  

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chroma_print::{print_info, print_success};
use dialoguer::{Confirm, Input, Select, console::Style, theme::ColorfulTheme};

const ENV_TEMPLATE_FILES: &[&str] = &[
    "example.env",
    ".example.env",
    "env.example",
    ".env.example",
    "env.sample",
    ".env.sample",
    ".env-dist",
];

#[derive(Debug)]
/// Configuration struct
pub struct Config {
    /// The name of the output file (e.g., .env)
    output_file: String,
    /// A hash map to store key-value pairs of environment variables
    vars: HashMap<String, String>,
}

impl Config {
    /// Getter for `output_file`
    pub fn output_file(&self) -> &str {
        return &self.output_file;
    }

    /// Sanitizes input by removing newlines, escaping backslashes and quotes, and trimming whitespace
    fn sanitize(input: &str) -> String {
        return input
            .replace(['\n', '\r'], "")
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .trim()
            .to_string();
    }

    /// Validates that a variable key follows standard ENV naming conventions
    pub fn is_key_valid(key: &str) -> bool {
        return !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_');
    }

    /// Returns a vector of sanitized (key, value) variables
    pub fn sanitized_vars(&self) -> Vec<(String, String)> {
        let mut sanitized: Vec<(String, String)> = self
            .vars
            .iter()
            .map(|(k, v)| (Self::sanitize(k), Self::sanitize(v)))
            .collect();

        sanitized.sort_by(|a, b| a.0.cmp(&b.0));

        return sanitized;
    }

    /// Initializes the configuration by prompting the user for input and optionally reading from template files
    fn init_config() -> Result<Option<Config>, Box<dyn Error>> {
        let theme: ColorfulTheme = ColorfulTheme {
            values_style: Style::new().yellow().dim(),
            ..ColorfulTheme::default()
        };
        print_success!("Welcome to env-gen! Let's create your configuration file:\n");

        let mut is_creating_new_config_from_template: bool = false;
        let mut vars: HashMap<String, String> = HashMap::new();
        let template_files_found: Vec<&str> = ENV_TEMPLATE_FILES
            .iter()
            .copied()
            .filter(|file| Path::new(file).exists())
            .collect();

        if !template_files_found.is_empty() {
            print_info!(
                "Found existing env template file(s): {}",
                template_files_found.join(", ")
            );
            is_creating_new_config_from_template = Confirm::with_theme(&theme)
            .with_prompt(
                "Do you want to create a new configuration file using one of the found template file(s) (If No/n, a blank file will be created)?",
            )
            .default(false)
            .interact()?;
        }

        if is_creating_new_config_from_template {
            let template_file_index = Select::with_theme(&theme)
                .with_prompt("Which template file do you want to use?")
                .default(0)
                .items(&template_files_found)
                .interact()?;

            let mut has_invalid_keys: bool = false;
            let template_file: &str = template_files_found[template_file_index];
            let contents: String = std::fs::read_to_string(template_file)?;
            for line in contents.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    if Config::is_key_valid(key) {
                        vars.entry(key.to_string()).or_insert(value.to_string());
                    } else {
                        has_invalid_keys = true;
                        print_info!("Skipping invalid key in template file: {}", key);
                    }
                }
            }

            if has_invalid_keys {
                print_info!(
                    "Keys must be non-empty and contain only alphanumeric characters or underscores."
                );
            }
        }

        let output_file: String = Input::with_theme(&theme)
            .with_prompt("Enter the name of the output file (default: .env):")
            .default(".env".into())
            .interact()?;

        let is_adding_vars_to_file: bool = Confirm::with_theme(&theme)
            .with_prompt("Do you want to add (key/value) variables?")
            .default(!is_creating_new_config_from_template)
            .interact()?;

        if is_adding_vars_to_file {
            loop {
                let key: String = Input::new().with_prompt("Key").interact_text()?;
                let value: String = Input::new().with_prompt("Value").interact_text()?;

                if !Config::is_key_valid(&key) {
                    print_info!(
                        "Invalid key: {}. Keys must be non-empty and contain only alphanumeric characters or underscores. Please try again.",
                        key
                    );
                    continue;
                }

                vars.entry(key).or_insert(value);

                let continue_input: bool = Confirm::new()
                    .with_prompt("Add another?")
                    .default(true)
                    .interact()?;

                if !continue_input {
                    break;
                }
            }
        }

        return Ok(Some(Config { output_file, vars }));
    }

    /// Main function to run the configuration initialization and create output file
    pub fn run() -> Result<(), Box<dyn Error>> {
        match Self::init_config() {
            Ok(Some(config)) => {
                let raw_output: &str = config.output_file();

                // Get the filename from the raw output path
                let file_name: &OsStr = Path::new(raw_output)
                    .file_name()
                    .ok_or("Invalid output filename")?;
                // Anchor the filename to CWD (for sanity)
                let mut full_path: PathBuf = std::env::current_dir()?;
                full_path.push(file_name);

                let file: File = File::create(&full_path)?;
                let mut writer: BufWriter<File> = BufWriter::new(file);

                for (key, value) in config.sanitized_vars() {
                    writeln!(writer, "{}=\"{}\"", key, value)?;
                }

                writer.flush()?;

                print_success!(
                    "\n{} file created successfully!",
                    file_name.to_string_lossy()
                );

                return Ok(());
            }
            Ok(None) => {
                print_info!("Aborted. No configuration file created.");
            }
            Err(error) => {
                return Err(error);
            }
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_key_valid() {
        // Valid keys
        assert!(Config::is_key_valid("DATABASE_URL"));
        assert!(Config::is_key_valid("PORT8080"));
        assert!(Config::is_key_valid("secret_key"));

        // Invalid keys
        assert!(!Config::is_key_valid(""));
        assert!(!Config::is_key_valid("KEY-WITH-DASH"));
        assert!(!Config::is_key_valid("KEY!@#"));
        assert!(!Config::is_key_valid("KEY SPACE"));
    }

    #[test]
    fn test_sanitize() {
        // Newline removal
        assert_eq!(Config::sanitize("hello\nworld\r"), "helloworld");

        // Escaping quotes and backslashes
        assert_eq!(
            Config::sanitize(r#"path\to\"file""#),
            r#"path\\to\\\"file\""#
        );

        // Trimming whitespace
        assert_eq!(Config::sanitize("  clean me  "), "clean me");
    }

    #[test]
    fn test_sanitized_vars_sorting_and_logic() {
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("KEY_1".to_string(), "value".to_string());
        vars.insert("KEY_2".to_string(), "value\nwith_newline".to_string());

        let config = Config {
            output_file: ".env".to_string(),
            vars,
        };

        let sanitized: Vec<(String, String)> = config.sanitized_vars();

        // Check sorted
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].0, "KEY_1");
        assert_eq!(sanitized[1].0, "KEY_2");

        // Check if value was sanitized
        assert_eq!(sanitized[0].1, "value");
        assert_eq!(sanitized[1].1, "valuewith_newline");
    }

    #[test]
    fn test_output_file_getter() {
        let config = Config {
            output_file: "test.env".to_string(),
            vars: HashMap::new(),
        };
        assert_eq!(config.output_file(), "test.env");
    }
}
