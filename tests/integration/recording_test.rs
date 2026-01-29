//! Unit tests for recording module

use agr::{Config, Recorder};

#[test]
fn generate_filename_has_correct_format() {
    let config = Config::default();
    let recorder = Recorder::new(config);
    let filename = recorder.generate_filename();
    assert!(filename.ends_with(".cast"));
    // New format: {directory}_{date}_{time}.cast
    assert!(filename.contains('_'));
}

#[test]
fn sanitize_filename_preserves_valid_chars() {
    assert_eq!(Recorder::sanitize_filename("my-session"), "my-session.cast");
    assert_eq!(Recorder::sanitize_filename("test_123"), "test_123.cast");
    assert_eq!(Recorder::sanitize_filename("file.cast"), "file.cast");
}

#[test]
fn sanitize_filename_replaces_spaces_with_dashes() {
    assert_eq!(Recorder::sanitize_filename("my session"), "my-session.cast");
    assert_eq!(Recorder::sanitize_filename("a b c"), "a-b-c.cast");
}

#[test]
fn sanitize_filename_replaces_special_chars() {
    assert_eq!(Recorder::sanitize_filename("test@#$%"), "test____.cast");
    assert_eq!(Recorder::sanitize_filename("file/name"), "file_name.cast");
}

#[test]
fn sanitize_filename_adds_extension() {
    assert_eq!(Recorder::sanitize_filename("session"), "session.cast");
}

#[test]
fn sanitize_filename_keeps_existing_extension() {
    assert_eq!(Recorder::sanitize_filename("session.cast"), "session.cast");
}
