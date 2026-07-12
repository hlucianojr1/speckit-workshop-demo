//! T041 (US5): digest correctness — the FNV-1a 64 implementation matches the textbook
//! definition (sandbox-scenes.spec.md §8), and two headless runs at the same seed
//! produce identical digest sequences.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use orbital_arena_rs::digest::{Fnv1a64, FNV_OFFSET_BASIS, FNV_PRIME};
use orbital_arena_rs::headless::{run_headless, HeadlessScene};

/// Independent textbook reference: h = basis; per byte: h ^= b, h *= prime (wrapping).
fn reference_fnv1a(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET_BASIS;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[test]
fn empty_digest_is_the_offset_basis() {
    assert_eq!(Fnv1a64::new().finish(), 0xcbf2_9ce4_8422_2325);
}

#[test]
fn constants_match_the_spec() {
    assert_eq!(FNV_OFFSET_BASIS, 0xcbf2_9ce4_8422_2325);
    assert_eq!(FNV_PRIME, 0x0000_0100_0000_01b3);
}

#[test]
fn mixing_a_double_hashes_its_bits_lsb_first() {
    // 1.5f64 has a known bit pattern; the spec mandates least-significant byte first.
    let value = 1.5f64;
    let expected = reference_fnv1a(&value.to_bits().to_le_bytes());

    let mut hash = Fnv1a64::new();
    hash.mix_f64(value);
    assert_eq!(hash.finish(), expected);
}

#[test]
fn mixing_zero_double_multiplies_basis_by_prime_eight_times() {
    // 0.0f64 is 8 zero bytes: each mix is h ^= 0 (no-op) then h *= prime.
    let mut expected = FNV_OFFSET_BASIS;
    for _ in 0..8 {
        expected = expected.wrapping_mul(FNV_PRIME);
    }
    let mut hash = Fnv1a64::new();
    hash.mix_f64(0.0);
    assert_eq!(hash.finish(), expected);
}

#[test]
fn digest_is_order_sensitive() {
    let mut ab = Fnv1a64::new();
    ab.mix_f64(1.0);
    ab.mix_f64(2.0);
    let mut ba = Fnv1a64::new();
    ba.mix_f64(2.0);
    ba.mix_f64(1.0);
    assert_ne!(ab.finish(), ba.finish());
}

#[test]
fn two_headless_runs_at_the_same_seed_are_byte_identical() {
    let dir = std::env::temp_dir();
    let path_a = dir.join("oars-digest-test-a.csv");
    let path_b = dir.join("oars-digest-test-b.csv");

    let report_a = run_headless(42, 60, &path_a, HeadlessScene::Rope).unwrap();
    let report_b = run_headless(42, 60, &path_b, HeadlessScene::Rope).unwrap();

    assert_eq!(report_a.digest, report_b.digest);
    let csv_a = std::fs::read(&path_a).unwrap();
    let csv_b = std::fs::read(&path_b).unwrap();
    assert_eq!(
        csv_a, csv_b,
        "same (seed, frames, scene) must be byte-identical"
    );

    let _ = std::fs::remove_file(path_a);
    let _ = std::fs::remove_file(path_b);
}

#[test]
fn different_seeds_produce_different_digests() {
    let dir = std::env::temp_dir();
    let path_a = dir.join("oars-digest-test-seed42.csv");
    let path_b = dir.join("oars-digest-test-seed7.csv");

    let report_a = run_headless(42, 60, &path_a, HeadlessScene::Rope).unwrap();
    let report_b = run_headless(7, 60, &path_b, HeadlessScene::Rope).unwrap();
    assert_ne!(report_a.digest, report_b.digest);

    let _ = std::fs::remove_file(path_a);
    let _ = std::fs::remove_file(path_b);
}
