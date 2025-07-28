//! Benchmarks for LazyTower proof verification to test O(1) claim

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Instant;

use lazytower_rs::{Digest, LazyTower, MembershipProof};

/// Mock digest for testing
#[derive(Clone, Debug, PartialEq, Eq)]
struct MockDigest;

impl Digest for MockDigest {
    type Output = Vec<u8>;

    fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output {
        let mut result = b"digest(".to_vec();
        result.extend_from_slice(item.as_ref());
        result.extend_from_slice(b")");
        result
    }

    fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output {
        let mut result = b"digest_items[".to_vec();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                result.extend_from_slice(b",");
            }
            result.extend_from_slice(item.as_ref());
        }
        result.extend_from_slice(b"]");
        result
    }
}

/// Helper function to create a tower with n items
fn create_tower_with_items(n: usize, width: usize) -> LazyTower<Vec<u8>, MockDigest> {
    let mut tower = LazyTower::new(width).unwrap();
    for i in 0..n {
        tower.append(format!("item_{}", i).into_bytes());
    }
    tower
}

/// Helper function to create proof for the middle item
fn create_proof_for_middle_item(
    tower: &LazyTower<Vec<u8>, MockDigest>,
) -> Option<MembershipProof<Vec<u8>, MockDigest>> {
    let middle_index = tower.len() / 2;
    tower.generate_proof(middle_index).ok()
}

/// Benchmark proof verification with different numbers of items
fn bench_verification_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_scaling");

    // Test with different numbers of items
    let sizes = vec![10, 50, 100, 500, 1000, 5000, 10000];
    let width = 4; // Standard width

    for size in sizes {
        let tower = create_tower_with_items(size, width);

        // Try to get a proof for the middle item
        if let Some(proof) = create_proof_for_middle_item(&tower) {
            group.bench_with_input(BenchmarkId::new("items", size), &proof, |b, proof| {
                b.iter(|| black_box(proof.verify()))
            });
        }
    }
    group.finish();
}

/// Benchmark verification with different tower widths
fn bench_verification_width_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_width_scaling");

    let widths = vec![2, 4, 8, 16, 32];
    let num_items = 1000;

    for width in widths {
        let tower = create_tower_with_items(num_items, width);

        if let Some(proof) = create_proof_for_middle_item(&tower) {
            group.bench_with_input(BenchmarkId::new("width", width), &proof, |b, proof| {
                b.iter(|| black_box(proof.verify()))
            });
        }
    }
    group.finish();
}

/// Manual timing test to measure verification cost scaling
fn manual_verification_timing_test() {
    println!("\n=== Manual Verification Timing Test ===");
    println!("Testing if verification cost remains constant as number of items increases");

    let sizes = vec![100, 500, 1000, 2000, 5000, 10000];
    let width = 4;
    let iterations = 1000; // Number of times to run verification for averaging

    println!("Size\tHeight\tAvg Time (ns)\tTime/Item (ns)");

    for size in sizes {
        let tower = create_tower_with_items(size, width);
        let height = tower.height();

        if let Some(proof) = create_proof_for_middle_item(&tower) {
            // Warm up
            for _ in 0..10 {
                proof.verify();
            }

            // Measure
            let start = Instant::now();
            for _ in 0..iterations {
                black_box(proof.verify());
            }
            let elapsed = start.elapsed();

            let avg_nanos = elapsed.as_nanos() / iterations as u128;
            let time_per_item = avg_nanos as f64 / size as f64;

            println!(
                "{}\t{}\t{}\t\t{:.3}",
                size, height, avg_nanos, time_per_item
            );
        } else {
            println!("{}\t-\tFailed to generate proof", size);
        }
    }
}

/// Test verification cost with different proof path lengths
fn bench_verification_path_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_path_length");

    // Create towers with different structures to get different path lengths
    let configs = vec![
        (100, 2),   // Deep tree (more levels)
        (100, 4),   // Medium depth
        (100, 8),   // Shallower tree
        (1000, 2),  // Very deep tree
        (1000, 16), // Very shallow tree
    ];

    for (size, width) in configs {
        let tower = create_tower_with_items(size, width);
        let height = tower.height();

        if let Some(proof) = create_proof_for_middle_item(&tower) {
            let path_len = proof.path.elements.len();
            group.bench_with_input(
                BenchmarkId::new("path_len", format!("{}_{}", path_len, height)),
                &proof,
                |b, proof| b.iter(|| black_box(proof.verify())),
            );
        }
    }
    group.finish();
}

/// Comprehensive analysis function
fn comprehensive_verification_analysis() {
    println!("\n=== Comprehensive Verification Analysis ===");

    let test_configs = vec![
        // (num_items, width, description)
        (50, 2, "Small dataset, narrow tree (deep)"),
        (50, 8, "Small dataset, wide tree (shallow)"),
        (500, 2, "Medium dataset, narrow tree"),
        (500, 8, "Medium dataset, wide tree"),
        (2000, 4, "Large dataset, standard width"),
        (5000, 2, "Very large dataset, narrow tree"),
        (5000, 16, "Very large dataset, wide tree"),
    ];

    println!("Config\t\t\t\tItems\tHeight\tPath Len\tTime (ns)\tTime/Log(n)");

    for (num_items, width, desc) in test_configs {
        let tower = create_tower_with_items(num_items, width);
        let height = tower.height();

        if let Some(proof) = create_proof_for_middle_item(&tower) {
            let path_len = proof.path.elements.len();

            // Measure verification time
            let iterations = 100;
            let start = Instant::now();
            for _ in 0..iterations {
                black_box(proof.verify());
            }
            let elapsed = start.elapsed();
            let avg_nanos = elapsed.as_nanos() / iterations as u128;

            let log_n = (num_items as f64).log2();
            let time_per_log = avg_nanos as f64 / log_n;

            println!(
                "{:<30}\t{}\t{}\t{}\t\t{}\t\t{:.1}",
                desc, num_items, height, path_len, avg_nanos, time_per_log
            );
        }
    }

    println!("\nAnalysis:");
    println!("- If verification is O(1), time should remain roughly constant");
    println!("- If verification is O(log n), time should grow with log(items)");
    println!("- Path length correlates with tree height, which grows as log_width(n)");
    println!("- Time/Log(n) should decrease if complexity is better than O(log n)");
}

// Criterion benchmark groups
criterion_group!(
    benches,
    bench_verification_scaling,
    bench_verification_width_scaling,
    bench_verification_path_length
);
criterion_main!(benches);

// Also run manual tests when executed directly
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_manual_timing_analysis() {
        manual_verification_timing_test();
        comprehensive_verification_analysis();
    }
}
