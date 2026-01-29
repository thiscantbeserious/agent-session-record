//! Config subcommands handler

use anyhow::Result;
use std::fs;
use std::io::{self, BufRead, Write};

use agr::config::migrate_config;
use agr::tui::current_theme;
use agr::tui::theme::ansi;
use agr::Config;

/// Show current configuration as TOML.
#[cfg(not(tarpaulin_include))]
pub fn handle_show() -> Result<()> {
    let config = Config::load()?;
    let toml_str = toml::to_string_pretty(&config)?;
    let theme = current_theme();
    println!("{}", theme.primary_text(&toml_str));
    Ok(())
}

/// Open configuration file in the default editor.
///
/// Uses $EDITOR environment variable (defaults to 'vi').
#[cfg(not(tarpaulin_include))]
pub fn handle_edit() -> Result<()> {
    let config_path = Config::config_path()?;
    let theme = current_theme();

    // Ensure config exists
    if !config_path.exists() {
        let config = Config::default();
        config.save()?;
    }

    // Get editor from environment
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    println!(
        "{}",
        theme.primary_text(&format!(
            "Opening {} with {}",
            config_path.display(),
            editor
        ))
    );

    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to open editor: {}", e))?;

    Ok(())
}

/// Migrate config file by adding missing fields.
///
/// Reads the existing config file (or empty if it doesn't exist),
/// adds any missing fields from the current default config,
/// shows a preview of changes, and prompts for confirmation.
#[cfg(not(tarpaulin_include))]
pub fn handle_migrate() -> Result<()> {
    let theme = current_theme();
    let config_path = Config::config_path()?;
    let file_exists = config_path.exists();

    // Read existing content (empty string if file doesn't exist)
    let content = if file_exists {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    // Run migration
    let result = migrate_config(&content)?;

    // Case 1: No changes needed
    if !result.has_changes() {
        println!("{}", theme.primary_text("Config is already up to date."));
        return Ok(());
    }

    // Case 2: Config file doesn't exist - offer to create with full defaults
    if !file_exists {
        println!(
            "{}",
            theme.primary_text("Config file does not exist. Will create with default settings.")
        );
        println!();
        print_diff_preview(&result.content, &[], true);
        println!();

        if !prompt_confirmation(&format!("Create {}?", config_path.display()))? {
            println!("{}", theme.primary_text("No changes made."));
            return Ok(());
        }

        // Create config directory and write file
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, &result.content)?;
        println!(
            "{}",
            theme.success_text("Config file created successfully.")
        );
        return Ok(());
    }

    // Case 3: Config exists but has missing fields - show diff and confirm
    let total_fields = result.added_fields.len();
    let total_sections = result.sections_added.len();

    // Print summary
    if total_sections > 0 {
        println!(
            "{}",
            theme.primary_text(&format!(
                "Found {} missing field(s) in {} new section(s):",
                total_fields, total_sections
            ))
        );
    } else {
        println!(
            "{}",
            theme.primary_text(&format!("Found {} missing field(s):", total_fields))
        );
    }
    println!();

    // Show diff preview - compare old content with new content
    print_diff_preview(&result.content, &result.added_fields, false);
    println!();

    // Prompt for confirmation
    if !prompt_confirmation(&format!(
        "Apply these changes to {}?",
        config_path.display()
    ))? {
        println!("{}", theme.primary_text("No changes made."));
        return Ok(());
    }

    // Write the updated config
    fs::write(&config_path, &result.content)?;
    println!("{}", theme.success_text("Config updated successfully."));

    Ok(())
}

/// Print a diff-style preview of the config changes.
///
/// Shows lines that contain added fields with a green `+` prefix.
/// For new files, shows all content as additions.
fn print_diff_preview(new_content: &str, added_fields: &[String], is_new_file: bool) {
    // Build a set of field names (without section prefix) for quick lookup
    let added_keys: std::collections::HashSet<&str> = added_fields
        .iter()
        .filter_map(|f| f.split('.').next_back())
        .collect();

    let mut current_section = String::new();
    let mut section_has_additions = false;
    let mut pending_section_header: Option<String> = None;

    for line in new_content.lines() {
        let trimmed = line.trim();

        // Track section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Check if this section was added
            let section_name = &trimmed[1..trimmed.len() - 1];
            let is_added_section = added_fields
                .iter()
                .any(|f| f.starts_with(&format!("{}.", section_name)));

            current_section = section_name.to_string();
            section_has_additions = is_added_section;

            if is_new_file || is_added_section {
                // For new files or added sections, print the header
                pending_section_header = Some(line.to_string());
            } else {
                pending_section_header = None;
            }
            continue;
        }

        // Check if this line is a field assignment
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();

            // Is this an added field?
            let is_added = added_keys.contains(key)
                && added_fields.contains(&format!("{}.{}", current_section, key));

            if is_new_file || is_added {
                // Print pending section header if we have one
                if let Some(header) = pending_section_header.take() {
                    println!("{}+{} {}{}", ansi::GREEN, ansi::RESET, ansi::GREEN, header);
                }

                // Print added line with green + prefix
                println!("{}+ {}{}", ansi::GREEN, line, ansi::RESET);
            } else if section_has_additions {
                // Show context lines in the section (without + prefix)
                // Only show the section header once we know there are additions
                if let Some(header) = pending_section_header.take() {
                    println!("  {}", header);
                }
                // Skip showing existing fields to keep diff focused
            }
        } else if is_new_file && !trimmed.is_empty() {
            // For new files, show comments too
            if let Some(header) = pending_section_header.take() {
                println!("{}+{} {}{}", ansi::GREEN, ansi::RESET, ansi::GREEN, header);
            }
            println!("{}+ {}{}", ansi::GREEN, line, ansi::RESET);
        }
    }
}

/// Prompt user for yes/no confirmation.
///
/// Returns true if user confirms (y/yes), false otherwise.
/// If stdin is not a TTY (non-interactive), returns false.
fn prompt_confirmation(message: &str) -> Result<bool> {
    let theme = current_theme();

    // Check if stdin is a TTY - if not, skip prompt and return false
    if !atty::is(atty::Stream::Stdin) {
        println!(
            "{}",
            theme.secondary_text("Non-interactive mode: use --yes to apply changes automatically")
        );
        return Ok(false);
    }

    print!("{} [y/N] ", theme.primary_text(message));
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}
