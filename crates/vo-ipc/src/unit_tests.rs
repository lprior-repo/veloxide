use crate::config::{
    parse_fd3_payload_as_argv, validate_program_path, validate_timeout, SubprocessConfig,
};
use crate::error::{ConfigError, IpcError};
use crate::run::{encode_fd4_payload, map_exit_code};
use crate::stderr::{
    finalize_capture, update_capture, StderrCapture, MAX_STDERR_BYTES, TRUNCATION_MARKER,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use tempfile::tempdir;

fn executable_file() -> std::path::PathBuf {
    let directory = tempdir().unwrap();
    let file = directory.path().join("fixture.sh");
    fs::write(&file, "#!/bin/sh\nexit 0\n").unwrap();
    let mut permissions = fs::metadata(&file).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&file, permissions).unwrap();
    let path = file.clone();
    std::mem::forget(directory);
    path
}

#[test]
fn new_accepts_executable_path() {
    let path = executable_file();
    let config = SubprocessConfig::new(&path, 1, b"ok".to_vec()).unwrap();
    assert_eq!(config.timeout_ms(), 1);
}

#[test]
fn new_rejects_zero_timeout() {
    let path = executable_file();
    let error = SubprocessConfig::new(&path, 0, Vec::new()).unwrap_err();
    assert_eq!(error, ConfigError::TimeoutMustBePositive { timeout_ms: 0 });
}

#[test]
fn new_rejects_missing_path() {
    let error = SubprocessConfig::new("/definitely/missing", 1, Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::ProgramMissing { .. }));
}

#[test]
fn new_rejects_non_executable_path() {
    let directory = tempdir().unwrap();
    let file = directory.path().join("fixture.sh");
    fs::write(&file, "#!/bin/sh\nexit 0\n").unwrap();
    let error = SubprocessConfig::new(&file, 1, Vec::new()).unwrap_err();
    assert!(matches!(error, ConfigError::ProgramNotExecutable { .. }));
}

#[test]
fn new_canonicalizes_path() {
    let path = executable_file();
    let config = SubprocessConfig::new(&path, 9, Vec::new()).unwrap();
    assert!(config.executable_path().is_absolute());
}

#[test]
fn new_preserves_payload() {
    let path = executable_file();
    let payload = b"alpha beta".to_vec();
    let config = SubprocessConfig::new(&path, 9, payload.clone()).unwrap();
    assert_eq!(config.fd3_payload(), payload.as_slice());
}

#[test]
fn validate_timeout_accepts_positive() {
    assert_eq!(validate_timeout(42), Ok(()));
}

#[test]
fn validate_timeout_rejects_zero() {
    assert_eq!(
        validate_timeout(0),
        Err(ConfigError::TimeoutMustBePositive { timeout_ms: 0 })
    );
}

#[test]
fn validate_program_path_accepts_executable() {
    let path = executable_file();
    assert_eq!(validate_program_path(&path), Ok(()));
}

#[test]
fn validate_program_path_rejects_missing() {
    let error = validate_program_path(std::path::Path::new("/missing/program")).unwrap_err();
    assert!(matches!(error, ConfigError::ProgramMissing { .. }));
}

#[test]
fn validate_program_path_rejects_non_executable() {
    let directory = tempdir().unwrap();
    let file = directory.path().join("plain.txt");
    fs::write(&file, "hello").unwrap();
    let error = validate_program_path(&file).unwrap_err();
    assert!(matches!(error, ConfigError::ProgramNotExecutable { .. }));
}

#[test]
fn parse_payload_as_argv_splits_ascii_whitespace() {
    let args = parse_fd3_payload_as_argv(b"echo  alpha\tbeta\n gamma");
    assert_eq!(args, vec!["echo", "alpha", "beta", "gamma"]);
}

#[test]
fn parse_payload_as_argv_returns_empty_for_invalid_utf8() {
    let args = parse_fd3_payload_as_argv(&[0xff, 0xfe]);
    assert!(args.is_empty());
}

#[test]
fn parse_payload_as_argv_returns_empty_for_blank_payload() {
    let args = parse_fd3_payload_as_argv(b"   \n\t");
    assert!(args.is_empty());
}

#[test]
fn parse_payload_as_argv_preserves_simple_tokens() {
    let args = parse_fd3_payload_as_argv(b"one two three");
    assert_eq!(args, vec!["one", "two", "three"]);
}

#[test]
fn update_capture_keeps_small_payload() {
    let capture = update_capture(StderrCapture::empty(), b"abc");
    assert_eq!(capture.bytes, b"abc");
    assert!(!capture.truncated);
}

#[test]
fn update_capture_marks_truncated_after_limit_crossed() {
    let seed = vec![b'x'; MAX_STDERR_BYTES];
    let capture = update_capture(StderrCapture::empty(), &seed);
    let truncated = update_capture(capture, b"y");
    assert!(truncated.truncated);
}

#[test]
fn update_capture_observed_bytes_counts_all_bytes() {
    let capture = update_capture(StderrCapture::empty(), b"abc");
    let updated = update_capture(capture, b"defg");
    assert_eq!(updated.observed_bytes, 7);
}

#[test]
fn update_capture_limits_preview_to_max() {
    let capture = update_capture(StderrCapture::empty(), &vec![b'x'; MAX_STDERR_BYTES + 99]);
    assert_eq!(capture.bytes.len(), MAX_STDERR_BYTES);
}

#[test]
fn finalize_capture_adds_marker_when_truncated() {
    let capture = StderrCapture {
        bytes: vec![b'x'; MAX_STDERR_BYTES],
        truncated: true,
        observed_bytes: MAX_STDERR_BYTES + 1,
    };
    let finalized = finalize_capture(capture);
    assert!(finalized.bytes.ends_with(TRUNCATION_MARKER.as_bytes()));
}

#[test]
fn finalize_capture_does_not_add_marker_when_not_truncated() {
    let capture = finalize_capture(StderrCapture {
        bytes: b"abc".to_vec(),
        truncated: false,
        observed_bytes: 3,
    });
    assert_eq!(capture.bytes, b"abc");
}

#[test]
fn finalize_capture_adds_marker_once() {
    let capture = StderrCapture {
        bytes: vec![b'x'; MAX_STDERR_BYTES],
        truncated: true,
        observed_bytes: MAX_STDERR_BYTES + 3,
    };
    let once = finalize_capture(capture.clone());
    let twice = finalize_capture(once.clone());
    assert_eq!(once.bytes, twice.bytes);
}

#[test]
fn finalize_capture_preserves_prefix() {
    let capture = StderrCapture {
        bytes: b"prefix".to_vec(),
        truncated: true,
        observed_bytes: 9,
    };
    let finalized = finalize_capture(capture);
    assert!(finalized.bytes.starts_with(b"prefix"));
}

#[test]
fn encode_fd4_payload_prefixes_big_endian_length() {
    let encoded = encode_fd4_payload(b"abc");
    assert_eq!(&encoded[..4], &[0, 0, 0, 3]);
}

#[test]
fn encode_fd4_payload_appends_payload_bytes() {
    let encoded = encode_fd4_payload(b"abc");
    assert_eq!(&encoded[4..], b"abc");
}

#[test]
fn encode_fd4_payload_handles_empty_payload() {
    let encoded = encode_fd4_payload(b"");
    assert_eq!(encoded, vec![0, 0, 0, 0]);
}

#[test]
fn map_exit_code_preserves_zero() {
    let status = std::process::ExitStatus::from_raw(0);
    assert_eq!(map_exit_code(status), 0);
}

#[test]
fn map_exit_code_preserves_non_zero() {
    let status = std::process::ExitStatus::from_raw(17 << 8);
    assert_eq!(map_exit_code(status), 17);
}

#[test]
fn map_exit_code_preserves_255() {
    let status = std::process::ExitStatus::from_raw(255 << 8);
    assert_eq!(map_exit_code(status), 255);
}

#[test]
fn map_exit_code_maps_sigterm_to_143() {
    let status = std::process::ExitStatus::from_raw(15);
    assert_eq!(map_exit_code(status), 143);
}

#[test]
fn map_exit_code_maps_sigkill_to_137() {
    let status = std::process::ExitStatus::from_raw(9);
    assert_eq!(map_exit_code(status), 137);
}

#[test]
fn config_error_timeout_equality_works() {
    assert_eq!(
        ConfigError::TimeoutMustBePositive { timeout_ms: 3 },
        ConfigError::TimeoutMustBePositive { timeout_ms: 3 }
    );
}

#[test]
fn ipc_error_timeout_contains_truncation_flag() {
    let error = IpcError::Timeout {
        elapsed_ms: 4,
        stderr_bytes: b"x".to_vec(),
        stderr_truncated: true,
    };
    assert!(matches!(
        error,
        IpcError::Timeout {
            stderr_truncated: true,
            ..
        }
    ));
}

#[test]
fn ipc_error_process_failed_contains_exit_code() {
    let error = IpcError::ProcessFailed {
        exit_code: 44,
        stderr_bytes: Vec::new(),
        stderr_truncated: false,
    };
    assert!(matches!(
        error,
        IpcError::ProcessFailed { exit_code: 44, .. }
    ));
}

#[test]
fn truncation_marker_matches_contract() {
    assert_eq!(TRUNCATION_MARKER, "\n[... TRUNCATED AT 1MB ...]");
}

#[test]
fn max_stderr_bytes_matches_contract() {
    assert_eq!(MAX_STDERR_BYTES, 1_048_576);
}
