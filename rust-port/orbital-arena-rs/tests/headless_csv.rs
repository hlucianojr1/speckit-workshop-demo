//! T042 (US5): headless CSV/stdout contract — exact header, row formats, and summary
//! line per sandbox-headless.spec.md §4.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use orbital_arena_rs::headless::{run_headless, HeadlessError, HeadlessScene};

fn run_to_string(seed: u64, frames: u32) -> (String, orbital_arena_rs::headless::HeadlessReport) {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "oars-csv-test-{seed}-{frames}-{}-{unique}.csv",
        std::process::id()
    ));
    let report = run_headless(seed, frames, &path, HeadlessScene::Rope).unwrap();
    let csv = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(path);
    (csv, report)
}

/// `%.9f`: optional minus, digits, '.', exactly nine fractional digits.
fn is_nine_decimal_float(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    match s.split_once('.') {
        Some((whole, frac)) => {
            !whole.is_empty()
                && whole.bytes().all(|b| b.is_ascii_digit())
                && frac.len() == 9
                && frac.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

#[test]
fn header_is_byte_exact() {
    let (csv, _) = run_to_string(42, 3);
    let header = csv.lines().next().unwrap();
    assert_eq!(
        header,
        "frame,sim_time,digest,rope0_x,rope0_y,ropeN_x,ropeN_y"
    );
}

#[test]
fn one_row_per_frame_with_reference_formats() {
    let frames = 5;
    let (csv, _) = run_to_string(42, frames);
    let rows: Vec<&str> = csv.lines().skip(1).collect();
    assert_eq!(rows.len(), frames as usize);

    for (i, row) in rows.iter().enumerate() {
        let cols: Vec<&str> = row.split(',').collect();
        assert_eq!(cols.len(), 7, "row {i} must have 7 columns: {row}");
        // frame: %u, 0-based
        assert_eq!(cols[0], i.to_string());
        // sim_time: %.9f, accumulating by 1/60 per frame
        assert!(
            is_nine_decimal_float(cols[1]),
            "sim_time format: {}",
            cols[1]
        );
        // digest: %016llx — exactly 16 lowercase hex digits
        assert_eq!(cols[2].len(), 16, "digest width: {}", cols[2]);
        assert!(cols[2]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        // four position columns: %.9f
        for col in &cols[3..7] {
            assert!(is_nine_decimal_float(col), "position format: {col}");
        }
    }

    // First row's sim_time is one fixed step (spec §3: row written AFTER the step).
    let first = rows[0].split(',').nth(1).unwrap();
    assert_eq!(first, "0.016666667");
}

#[test]
fn digest_column_matches_final_report_digest_on_last_row() {
    let (csv, report) = run_to_string(42, 4);
    let last_row = csv.lines().last().unwrap();
    let digest_col = last_row.split(',').nth(2).unwrap();
    assert_eq!(digest_col, format!("{:016x}", report.digest));
}

#[test]
fn summary_line_matches_reference_stdout_format() {
    let (_, report) = run_to_string(42, 2);
    let line = report.summary_line();
    assert_eq!(
        line,
        format!("trace_digest={:016x} frames=2 scene=rope", report.digest)
    );
}

#[test]
fn zero_frames_is_an_argument_error() {
    let path = std::env::temp_dir().join("oars-csv-test-zero.csv");
    let result = run_headless(42, 0, &path, HeadlessScene::Rope);
    assert!(matches!(result, Err(HeadlessError::InvalidArguments)));
}

#[test]
fn sim_advances_across_frames() {
    // The digest must change frame-over-frame while the sim is animating.
    let (csv, _) = run_to_string(42, 3);
    let digests: Vec<&str> = csv
        .lines()
        .skip(1)
        .map(|row| row.split(',').nth(2).unwrap())
        .collect();
    assert_ne!(digests[0], digests[1]);
    assert_ne!(digests[1], digests[2]);
}
