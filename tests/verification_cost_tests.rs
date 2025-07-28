//! Tests to verify the O(1) claim for LazyTower proof verification

use lazytower_rs::{Digest, LazyTower};
use std::time::Instant;

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

/// Create a tower with specified number of items
fn create_test_tower(num_items: usize, width: usize) -> LazyTower<Vec<u8>, MockDigest> {
    let mut tower = LazyTower::new(width).unwrap();
    for i in 0..num_items {
        tower.append(format!("item_{}", i).into_bytes());
    }
    tower
}

#[test]
fn test_verification_time_scaling() {
    println!("\n=== Verification Time Scaling Test ===");
    println!("Testing if verification time remains constant as dataset grows");

    let test_sizes = vec![100, 200, 500, 1000, 2000, 5000];
    let width = 4;
    let measurement_rounds = 50;

    println!("Items\tHeight\tAvg Time (ns)\tStd Dev\t\tTime/Item");

    let mut results = Vec::new();

    for &size in &test_sizes {
        let tower = create_test_tower(size, width);
        let height = tower.height();

        // Test verification time for middle item (worst case in balanced tree)
        let middle_index = size / 2;

        match tower.generate_proof(middle_index) {
            Ok(proof) => {
                // Warm up
                for _ in 0..5 {
                    proof.verify();
                }

                // Collect timing measurements
                let mut times = Vec::new();
                for _ in 0..measurement_rounds {
                    let start = Instant::now();
                    proof.verify();
                    let elapsed = start.elapsed().as_nanos();
                    times.push(elapsed);
                }

                // Calculate statistics
                let avg_time = times.iter().sum::<u128>() / times.len() as u128;
                let variance = times
                    .iter()
                    .map(|&t| ((t as i128) - (avg_time as i128)).pow(2) as u128)
                    .sum::<u128>()
                    / times.len() as u128;
                let std_dev = (variance as f64).sqrt();
                let time_per_item = avg_time as f64 / size as f64;

                println!(
                    "{}\t{}\t{}\t\t{:.1}\t\t{:.3}",
                    size, height, avg_time, std_dev, time_per_item
                );

                results.push((size, height, avg_time, std_dev));
            }
            Err(_) => {
                println!("{}\t{}\tFailed to generate proof", size, height);
            }
        }
    }

    // Analyze results
    if results.len() >= 2 {
        println!("\n=== Analysis ===");
        let first = results[0];
        let last = results[results.len() - 1];

        let size_ratio = last.0 as f64 / first.0 as f64;
        let time_ratio = last.2 as f64 / first.2 as f64;
        let height_ratio = last.1 as f64 / first.1 as f64;

        println!("Dataset size increased by: {:.1}x", size_ratio);
        println!("Tree height increased by: {:.1}x", height_ratio);
        println!("Verification time increased by: {:.2}x", time_ratio);

        if time_ratio < 1.5 {
            println!("✓ Verification time appears to be roughly constant (O(1))");
        } else if time_ratio < size_ratio.log2() * 1.5 {
            println!("? Verification time grows sub-logarithmically");
        } else if time_ratio < size_ratio.log2() * 2.0 {
            println!("⚠ Verification time appears to be O(log n)");
        } else {
            println!("✗ Verification time grows faster than O(log n)");
        }
    }
}

#[test]
fn test_verification_complexity_analysis() {
    println!("\n=== Verification Complexity Analysis ===");

    // Test with different widths to see how tree structure affects verification
    let test_configs = vec![
        (1000, 2, "Deep tree (width=2)"),
        (1000, 4, "Standard tree (width=4)"),
        (1000, 8, "Wide tree (width=8)"),
        (1000, 16, "Very wide tree (width=16)"),
    ];

    println!("Configuration\t\t\tHeight\tPath Len\tTime (ns)\tOps Count");

    for (size, width, desc) in test_configs {
        let tower = create_test_tower(size, width);
        let height = tower.height();

        if let Ok(proof) = tower.generate_proof(size / 2) {
            let path_length = proof.path.elements.len();

            // Measure verification time
            let iterations = 100;
            let start = Instant::now();
            for _ in 0..iterations {
                proof.verify();
            }
            let avg_time = start.elapsed().as_nanos() / iterations as u128;

            // Estimate operation count based on path traversal
            let mut estimated_ops = 0;
            for element in &proof.path.elements {
                match element {
                    lazytower_rs::proof::PathElement::Siblings { siblings, .. } => {
                        estimated_ops += siblings.len() + 1; // Combining operations
                    }
                    lazytower_rs::proof::PathElement::RawSiblings { siblings, .. } => {
                        estimated_ops += siblings.len() + 1; // Digest operations
                    }
                }
            }

            println!(
                "{}\t{}\t{}\t\t{}\t\t{}",
                desc, height, path_length, avg_time, estimated_ops
            );
        }
    }

    println!("\nNote: If verification is truly O(1), operation count should not");
    println!("depend on tree height or number of items in the tower.");
}

#[test]
fn test_proof_path_operations_count() {
    println!("\n=== Proof Path Operations Analysis ===");
    println!("Analyzing the actual operations performed during verification");

    let sizes = vec![50, 100, 500, 1000, 5000];
    let width = 4;

    println!("Items\tHeight\tPath Ops\tDigest Ops\tCombine Ops");

    for size in sizes {
        let tower = create_test_tower(size, width);
        let height = tower.height();

        if let Ok(proof) = tower.generate_proof(size / 2) {
            let mut path_traversals = 0;
            let mut digest_operations = 0;
            let mut combine_operations = 0;

            // Count operations in the proof path
            for element in &proof.path.elements {
                path_traversals += 1;

                match element {
                    lazytower_rs::proof::PathElement::Siblings { siblings, .. } => {
                        // Each sibling needs to be combined
                        combine_operations += siblings.len();
                        digest_operations += 1; // Final digest computation
                    }
                    lazytower_rs::proof::PathElement::RawSiblings { siblings, .. } => {
                        // Each sibling needs to be digested first, then combined
                        digest_operations += siblings.len() + 1;
                    }
                }
            }

            println!(
                "{}\t{}\t{}\t\t{}\t\t{}",
                size, height, path_traversals, digest_operations, combine_operations
            );
        }
    }

    println!("\nTrue O(1) verification would show constant operation counts");
    println!("regardless of dataset size.");
}

#[test]
fn test_constant_time_verification_hypothesis() {
    println!("\n=== Constant Time Verification Hypothesis Test ===");

    // Create multiple towers with the same height but different number of items
    // by using different widths
    let target_height = 3;
    let test_cases = vec![
        // (approximate_items, width, description)
        (64, 4, "4^3 = 64 items, width=4"),
        (125, 5, "5^3 = 125 items, width=5"),
        (216, 6, "6^3 = 216 items, width=6"),
        (343, 7, "7^3 = 343 items, width=7"),
    ];

    println!("Test Case\t\t\tActual Items\tHeight\tTime (ns)");

    let mut times = Vec::new();

    for (target_items, width, desc) in test_cases {
        // Create tower close to target items
        let actual_items = std::cmp::min(target_items, 1000); // Limit for test performance
        let tower = create_test_tower(actual_items, width);
        let actual_height = tower.height();

        if let Ok(proof) = tower.generate_proof(actual_items / 2) {
            // Measure verification time
            let iterations = 200;
            let start = Instant::now();
            for _ in 0..iterations {
                proof.verify();
            }
            let avg_time = start.elapsed().as_nanos() / iterations as u128;

            println!(
                "{}\t{}\t\t{}\t\t{}",
                desc, actual_items, actual_height, avg_time
            );

            if actual_height <= target_height + 1 {
                // Allow some variance
                times.push(avg_time);
            }
        }
    }

    // Analyze variance in times for similar tree heights
    if times.len() >= 2 {
        let avg = times.iter().sum::<u128>() / times.len() as u128;
        let max_time = *times.iter().max().unwrap();
        let min_time = *times.iter().min().unwrap();
        let variance_ratio = max_time as f64 / min_time as f64;

        println!("\nTiming Analysis:");
        println!("Average time: {} ns", avg);
        println!("Time range: {} - {} ns", min_time, max_time);
        println!("Variance ratio: {:.2}x", variance_ratio);

        if variance_ratio < 2.0 {
            println!("✓ Low variance suggests O(1) behavior");
        } else {
            println!("⚠ High variance suggests complexity depends on tree structure");
        }
    }
}
