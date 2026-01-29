//! Tests for filename sanitization and generation.
//!
//! These tests are written BEFORE implementation (TDD approach).

use agr::files::filename::{self, Config, FilenameError, Template, TemplateError};

// ============================================================================
// Space Replacement Tests
// ============================================================================

#[test]
fn sanitize_replaces_spaces_with_hyphens() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my project", &config), "my-project");
}

#[test]
fn sanitize_replaces_multiple_spaces() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my   project", &config), "my-project");
}

#[test]
fn sanitize_replaces_tabs_with_hyphens() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my\tproject", &config), "my-project");
}

#[test]
fn sanitize_replaces_mixed_whitespace() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my \t\n project", &config), "my-project");
}

// ============================================================================
// Invalid Character Removal Tests
// ============================================================================

#[test]
fn sanitize_removes_forward_slash() {
    let config = Config::default();
    assert_eq!(filename::sanitize("path/to/file", &config), "pathtofile");
}

#[test]
fn sanitize_removes_backslash() {
    let config = Config::default();
    assert_eq!(filename::sanitize("path\\to\\file", &config), "pathtofile");
}

#[test]
fn sanitize_removes_colon() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file:name", &config), "filename");
}

#[test]
fn sanitize_removes_asterisk() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file*name", &config), "filename");
}

#[test]
fn sanitize_removes_question_mark() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file?name", &config), "filename");
}

#[test]
fn sanitize_removes_quotes() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file\"name", &config), "filename");
}

#[test]
fn sanitize_removes_angle_brackets() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file<name>", &config), "filename");
}

#[test]
fn sanitize_removes_pipe() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file|name", &config), "filename");
}

#[test]
fn sanitize_removes_all_invalid_chars() {
    let config = Config::default();
    assert_eq!(
        filename::sanitize("a/b\\c:d*e?f\"g<h>i|j", &config),
        "abcdefghij"
    );
}

// ============================================================================
// Unicode Handling Tests
// ============================================================================

#[test]
fn sanitize_transliterates_accented_chars() {
    let config = Config::default();
    assert_eq!(filename::sanitize("café", &config), "cafe");
}

#[test]
fn sanitize_transliterates_umlauts() {
    let config = Config::default();
    assert_eq!(filename::sanitize("über", &config), "uber");
}

#[test]
fn sanitize_removes_non_transliteratable_unicode() {
    let config = Config::default();
    // Japanese characters that can't be transliterated to ASCII
    let result = filename::sanitize("日本語", &config);
    // Should either be empty (then fallback) or removed
    assert!(!result.contains('日'));
}

#[test]
fn sanitize_handles_emoji() {
    let config = Config::default();
    let result = filename::sanitize("project🚀name", &config);
    assert!(!result.contains('🚀'));
    // Should preserve the ASCII parts
    assert!(result.contains("project"));
    assert!(result.contains("name"));
}

#[test]
fn sanitize_handles_mixed_unicode_and_ascii() {
    let config = Config::default();
    let result = filename::sanitize("my-projeçt_v2", &config);
    assert_eq!(result, "my-project_v2");
}

// ============================================================================
// Leading/Trailing Trimming Tests
// ============================================================================

#[test]
fn sanitize_trims_leading_spaces() {
    let config = Config::default();
    assert_eq!(filename::sanitize("  project", &config), "project");
}

#[test]
fn sanitize_trims_trailing_spaces() {
    let config = Config::default();
    assert_eq!(filename::sanitize("project  ", &config), "project");
}

#[test]
fn sanitize_trims_leading_dots() {
    let config = Config::default();
    assert_eq!(filename::sanitize("..project", &config), "project");
}

#[test]
fn sanitize_trims_trailing_dots() {
    let config = Config::default();
    assert_eq!(filename::sanitize("project..", &config), "project");
}

#[test]
fn sanitize_trims_mixed_leading_chars() {
    let config = Config::default();
    assert_eq!(filename::sanitize(". . .project", &config), "project");
}

#[test]
fn sanitize_trims_leading_hyphens() {
    let config = Config::default();
    assert_eq!(filename::sanitize("---project", &config), "project");
}

#[test]
fn sanitize_trims_trailing_hyphens() {
    let config = Config::default();
    assert_eq!(filename::sanitize("project---", &config), "project");
}

// ============================================================================
// Windows Reserved Names Tests
// ============================================================================

#[test]
fn sanitize_handles_con() {
    let config = Config::default();
    assert_eq!(filename::sanitize("CON", &config), "_CON");
}

#[test]
fn sanitize_handles_prn() {
    let config = Config::default();
    assert_eq!(filename::sanitize("PRN", &config), "_PRN");
}

#[test]
fn sanitize_handles_aux() {
    let config = Config::default();
    assert_eq!(filename::sanitize("AUX", &config), "_AUX");
}

#[test]
fn sanitize_handles_nul() {
    let config = Config::default();
    assert_eq!(filename::sanitize("NUL", &config), "_NUL");
}

#[test]
fn sanitize_handles_com_ports() {
    let config = Config::default();
    assert_eq!(filename::sanitize("COM1", &config), "_COM1");
    assert_eq!(filename::sanitize("COM9", &config), "_COM9");
}

#[test]
fn sanitize_handles_lpt_ports() {
    let config = Config::default();
    assert_eq!(filename::sanitize("LPT1", &config), "_LPT1");
    assert_eq!(filename::sanitize("LPT9", &config), "_LPT9");
}

#[test]
fn sanitize_handles_reserved_names_case_insensitive() {
    let config = Config::default();
    assert_eq!(filename::sanitize("con", &config), "_con");
    assert_eq!(filename::sanitize("Con", &config), "_Con");
}

#[test]
fn sanitize_allows_reserved_names_as_substrings() {
    let config = Config::default();
    // "CONTROLLER" contains "CON" but should not be treated as reserved
    assert_eq!(filename::sanitize("CONTROLLER", &config), "CONTROLLER");
}

#[test]
fn sanitize_handles_reserved_names_with_extensions() {
    let config = Config::default();
    // Reserved names with extensions should also be prefixed
    assert_eq!(filename::sanitize("CON.txt", &config), "_CON.txt");
    assert_eq!(filename::sanitize("NUL.cast", &config), "_NUL.cast");
    assert_eq!(filename::sanitize("PRN.doc", &config), "_PRN.doc");
}

// ============================================================================
// Empty Result Fallback Tests
// ============================================================================

#[test]
fn sanitize_empty_string_returns_fallback() {
    let config = Config::default();
    assert_eq!(filename::sanitize("", &config), "recording");
}

#[test]
fn sanitize_only_spaces_returns_fallback() {
    let config = Config::default();
    assert_eq!(filename::sanitize("   ", &config), "recording");
}

#[test]
fn sanitize_only_invalid_chars_returns_fallback() {
    let config = Config::default();
    assert_eq!(filename::sanitize("/\\:*?\"<>|", &config), "recording");
}

#[test]
fn sanitize_only_dots_returns_fallback() {
    let config = Config::default();
    assert_eq!(filename::sanitize("...", &config), "recording");
}

#[test]
fn sanitize_transliterates_cjk_characters() {
    let config = Config::default();
    // CJK characters get romanized by deunicode
    let result = filename::sanitize("日本語", &config);
    // deunicode romanizes Japanese characters
    assert!(!result.contains('日'));
    assert!(!result.is_empty());
}

// ============================================================================
// Directory Truncation Tests
// ============================================================================

#[test]
fn sanitize_directory_truncates_to_max_length() {
    let config = Config {
        directory_max_length: 10,
    };
    let long_name = "this-is-a-very-long-directory-name";
    let result = filename::sanitize_directory(long_name, &config);
    assert_eq!(result.len(), 10);
    assert_eq!(result, "this-is-a-");
}

#[test]
fn sanitize_directory_preserves_short_names() {
    let config = Config {
        directory_max_length: 50,
    };
    let result = filename::sanitize_directory("short", &config);
    assert_eq!(result, "short");
}

#[test]
fn sanitize_directory_truncates_after_sanitization() {
    let config = Config {
        directory_max_length: 10,
    };
    // Spaces become hyphens, then truncate
    let result = filename::sanitize_directory("my long project name", &config);
    assert!(result.len() <= 10);
}

#[test]
fn sanitize_directory_default_max_is_50() {
    let config = Config::default();
    assert_eq!(config.directory_max_length, 50);
}

#[test]
fn config_new_enforces_minimum_directory_length() {
    // Config::new should enforce minimum of 1
    let config = Config::new(0);
    assert_eq!(config.directory_max_length, 1);

    let config = Config::new(5);
    assert_eq!(config.directory_max_length, 5);
}

// ============================================================================
// Final Length Validation Tests
// ============================================================================

#[test]
fn validate_length_accepts_short_filename() {
    assert!(filename::validate_length("short.cast").is_ok());
}

#[test]
fn validate_length_accepts_255_chars() {
    let name = "a".repeat(255);
    assert!(filename::validate_length(&name).is_ok());
}

#[test]
fn validate_length_rejects_256_chars() {
    let name = "a".repeat(256);
    let result = filename::validate_length(&name);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        FilenameError::TooLong {
            length: 256,
            max: 255
        }
    );
}

#[test]
fn validate_length_rejects_very_long_filename() {
    let name = "a".repeat(1000);
    let result = filename::validate_length(&name);
    assert!(result.is_err());
}

// ============================================================================
// Preservation Tests (things that should NOT change)
// ============================================================================

#[test]
fn sanitize_preserves_alphanumeric() {
    let config = Config::default();
    assert_eq!(filename::sanitize("abc123XYZ", &config), "abc123XYZ");
}

#[test]
fn sanitize_preserves_hyphens() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my-project", &config), "my-project");
}

#[test]
fn sanitize_preserves_underscores() {
    let config = Config::default();
    assert_eq!(filename::sanitize("my_project", &config), "my_project");
}

#[test]
fn sanitize_preserves_dots_in_middle() {
    let config = Config::default();
    assert_eq!(filename::sanitize("file.v2.0", &config), "file.v2.0");
}

// ============================================================================
// Combined/Integration Tests
// ============================================================================

#[test]
fn sanitize_handles_realistic_directory_name() {
    let config = Config::default();
    // Realistic example: "My Project (v2)"
    assert_eq!(
        filename::sanitize("My Project (v2)", &config),
        "My-Project-v2"
    );
}

#[test]
fn sanitize_handles_path_like_input() {
    let config = Config::default();
    // User might accidentally pass a path - slashes removed, space becomes hyphen
    assert_eq!(
        filename::sanitize("/home/user/my project", &config),
        "homeusermy-project"
    );
}

#[test]
fn sanitize_collapses_multiple_hyphens() {
    let config = Config::default();
    // Multiple spaces or hyphens should collapse to single hyphen
    assert_eq!(filename::sanitize("my---project", &config), "my-project");
    assert_eq!(filename::sanitize("my   project", &config), "my-project");
}

// ============================================================================
// Template Parsing Tests
// ============================================================================

#[test]
fn template_parse_literal_only() {
    let template = Template::parse("my-recording").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_directory_tag() {
    let template = Template::parse("{directory}").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_date_tag_default_format() {
    let template = Template::parse("{date}").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_date_tag_custom_format() {
    let template = Template::parse("{date:%Y-%m-%d}").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_time_tag_default_format() {
    let template = Template::parse("{time}").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_time_tag_custom_format() {
    let template = Template::parse("{time:%H%M%S}").unwrap();
    assert_eq!(template.segments().len(), 1);
}

#[test]
fn template_parse_mixed_tags_and_literals() {
    // Default template: {directory}_{date:%y%m%d}_{time:%H%M}
    let template = Template::parse("{directory}_{date:%y%m%d}_{time:%H%M}").unwrap();
    // Should have: directory, literal "_", date, literal "_", time
    assert_eq!(template.segments().len(), 5);
}

#[test]
fn template_parse_literal_at_start() {
    let template = Template::parse("prefix-{directory}").unwrap();
    assert_eq!(template.segments().len(), 2);
}

#[test]
fn template_parse_literal_at_end() {
    let template = Template::parse("{directory}-suffix").unwrap();
    assert_eq!(template.segments().len(), 2);
}

#[test]
fn template_parse_empty_returns_error() {
    let result = Template::parse("");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TemplateError::Empty));
}

#[test]
fn template_parse_unclosed_brace_returns_error() {
    let result = Template::parse("{directory");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TemplateError::UnclosedBrace));
}

#[test]
fn template_parse_unknown_tag_returns_error() {
    let result = Template::parse("{unknown}");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TemplateError::UnknownTag(_)));
}

#[test]
fn template_parse_invalid_format_string_returns_error() {
    // Invalid strftime format (empty after colon)
    let result = Template::parse("{date:}");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TemplateError::InvalidFormat(_)
    ));
}

#[test]
fn template_parse_format_without_specifiers_returns_error() {
    // Format with no valid strftime specifiers
    let result = Template::parse("{date:invalid}");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TemplateError::InvalidFormat(_)
    ));
}

#[test]
fn template_parse_unmatched_close_brace_returns_error() {
    let result = Template::parse("test}bar");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TemplateError::UnmatchedCloseBrace
    ));
}

#[test]
fn template_default_constant_exists() {
    let template = Template::default();
    // Should parse the default template successfully
    assert!(!template.segments().is_empty());
}

#[test]
fn template_parse_nested_braces_returns_error() {
    let result = Template::parse("{date:{%Y}}");
    assert!(result.is_err());
}

#[test]
fn template_parse_only_literal_underscore() {
    let template = Template::parse("_").unwrap();
    assert_eq!(template.segments().len(), 1);
}

// ============================================================================
// Template Rendering Tests
// ============================================================================

#[test]
fn template_render_literal_only() {
    let template = Template::parse("my-recording").unwrap();
    let config = Config::default();
    let result = template.render("test-dir", &config);
    assert_eq!(result, "my-recording");
}

#[test]
fn template_render_directory_tag() {
    let template = Template::parse("{directory}").unwrap();
    let config = Config::default();
    let result = template.render("my-project", &config);
    assert_eq!(result, "my-project");
}

#[test]
fn template_render_directory_sanitized() {
    let template = Template::parse("{directory}").unwrap();
    let config = Config::default();
    // Directory with spaces should be sanitized
    let result = template.render("My Project", &config);
    assert_eq!(result, "My-Project");
}

#[test]
fn template_render_directory_truncated() {
    let template = Template::parse("{directory}").unwrap();
    let config = Config {
        directory_max_length: 10,
    };
    let result = template.render("very-long-directory-name", &config);
    assert_eq!(result.len(), 10);
}

#[test]
fn template_render_date_default_format() {
    let template = Template::parse("{date}").unwrap();
    let config = Config::default();
    let result = template.render("dir", &config);
    // Default format is %y%m%d (6 digits)
    assert_eq!(result.len(), 6);
    assert!(result.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn template_render_time_default_format() {
    let template = Template::parse("{time}").unwrap();
    let config = Config::default();
    let result = template.render("dir", &config);
    // Default format is %H%M (4 digits)
    assert_eq!(result.len(), 4);
    assert!(result.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn template_render_date_custom_format() {
    let template = Template::parse("{date:%Y}").unwrap();
    let config = Config::default();
    let result = template.render("dir", &config);
    // Should be 4-digit year
    assert_eq!(result.len(), 4);
    assert!(result.starts_with("20")); // 21st century
}

#[test]
fn template_render_full_default_template() {
    let template = Template::default();
    let config = Config::default();
    let result = template.render("my-project", &config);
    // Should contain directory, underscore separators, date, time
    assert!(result.contains("my-project"));
    assert!(result.contains('_'));
}

#[test]
fn template_render_preserves_literal_separators() {
    let template = Template::parse("{directory}--{date}").unwrap();
    let config = Config::default();
    let result = template.render("test", &config);
    assert!(result.contains("--"));
}

// ============================================================================
// Generate Function Tests
// ============================================================================

#[test]
fn generate_returns_filename_with_cast_extension() {
    let config = Config::default();
    let result = filename::generate("my-project", "{directory}", &config).unwrap();
    assert!(result.ends_with(".cast"));
}

#[test]
fn generate_uses_template() {
    let config = Config::default();
    let result = filename::generate("test-dir", "{directory}", &config).unwrap();
    assert_eq!(result, "test-dir.cast");
}

#[test]
fn generate_sanitizes_directory() {
    let config = Config::default();
    let result = filename::generate("My Project", "{directory}", &config).unwrap();
    assert_eq!(result, "My-Project.cast");
}

#[test]
fn generate_with_default_template() {
    let config = Config::default();
    let result = filename::generate(
        "my-project",
        "{directory}_{date:%y%m%d}_{time:%H%M}",
        &config,
    )
    .unwrap();
    assert!(result.starts_with("my-project_"));
    assert!(result.ends_with(".cast"));
}

#[test]
fn generate_validates_final_length() {
    let config = Config {
        directory_max_length: 300, // Allow long directory
    };
    // Create a template that would produce a very long filename
    let long_dir = "a".repeat(260);
    let result = filename::generate(&long_dir, "{directory}", &config);
    // Should fail because final filename > 255 chars
    assert!(result.is_err());
}

#[test]
fn generate_with_invalid_template_returns_error() {
    let config = Config::default();
    let result = filename::generate("dir", "{unknown}", &config);
    assert!(result.is_err());
}
