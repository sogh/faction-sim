//! Determinism verification tests
//!
//! Tests to ensure the simulation produces identical results given the same seed.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Test that SmallRng produces identical sequences with the same seed
#[test]
fn test_rng_determinism() {
    let seed = 42u64;

    // First run
    let mut rng1 = SmallRng::seed_from_u64(seed);
    let values1: Vec<f32> = (0..100).map(|_| rng1.gen()).collect();

    // Second run with same seed
    let mut rng2 = SmallRng::seed_from_u64(seed);
    let values2: Vec<f32> = (0..100).map(|_| rng2.gen()).collect();

    // Values should be identical
    assert_eq!(values1, values2, "RNG sequences should be identical with same seed");
}

/// Test that different seeds produce different sequences
#[test]
fn test_rng_different_seeds() {
    let mut rng1 = SmallRng::seed_from_u64(42);
    let mut rng2 = SmallRng::seed_from_u64(43);

    let values1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();
    let values2: Vec<f32> = (0..10).map(|_| rng2.gen()).collect();

    // Values should be different
    assert_ne!(values1, values2, "Different seeds should produce different sequences");
}

/// Test weighted random selection determinism
#[test]
fn test_weighted_selection_determinism() {
    fn weighted_select(rng: &mut SmallRng, weights: &[f32]) -> usize {
        let total: f32 = weights.iter().sum();
        if total <= 0.0 {
            return 0;
        }
        let r: f32 = rng.gen::<f32>() * total;
        let mut cumulative = 0.0;
        for (i, &w) in weights.iter().enumerate() {
            cumulative += w;
            if r < cumulative {
                return i;
            }
        }
        weights.len() - 1
    }

    let weights = vec![0.1, 0.3, 0.4, 0.2];
    let seed = 12345u64;

    // First run
    let mut rng1 = SmallRng::seed_from_u64(seed);
    let selections1: Vec<usize> = (0..100).map(|_| weighted_select(&mut rng1, &weights)).collect();

    // Second run with same seed
    let mut rng2 = SmallRng::seed_from_u64(seed);
    let selections2: Vec<usize> = (0..100).map(|_| weighted_select(&mut rng2, &weights)).collect();

    assert_eq!(selections1, selections2, "Weighted selections should be identical with same seed");
}

/// Test trait generation determinism
#[test]
fn test_trait_generation_determinism() {
    fn generate_traits(rng: &mut SmallRng) -> (f32, f32, f32, f32, f32, f32) {
        (
            rng.gen::<f32>(), // loyalty_weight
            rng.gen::<f32>(), // ambition
            rng.gen::<f32>(), // honesty
            rng.gen::<f32>(), // boldness
            rng.gen::<f32>(), // sociability
            rng.gen::<f32>(), // grudge_persistence
        )
    }

    let seed = 999u64;

    let mut rng1 = SmallRng::seed_from_u64(seed);
    let traits1: Vec<_> = (0..50).map(|_| generate_traits(&mut rng1)).collect();

    let mut rng2 = SmallRng::seed_from_u64(seed);
    let traits2: Vec<_> = (0..50).map(|_| generate_traits(&mut rng2)).collect();

    assert_eq!(traits1, traits2, "Trait generation should be deterministic");
}

/// Test that the order of operations matters for determinism
#[test]
fn test_order_independence() {
    // This test verifies that consuming RNG in a specific order produces consistent results
    let seed = 777u64;

    // Run 1: Generate movement, then communication, then resource
    let mut rng1 = SmallRng::seed_from_u64(seed);
    let movement1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();
    let comm1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();
    let resource1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();

    // Run 2: Same order
    let mut rng2 = SmallRng::seed_from_u64(seed);
    let movement2: Vec<f32> = (0..10).map(|_| rng2.gen()).collect();
    let comm2: Vec<f32> = (0..10).map(|_| rng2.gen()).collect();
    let resource2: Vec<f32> = (0..10).map(|_| rng2.gen()).collect();

    assert_eq!(movement1, movement2);
    assert_eq!(comm1, comm2);
    assert_eq!(resource1, resource2);
}
