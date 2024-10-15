use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Function to generate a random number based on a string input
/// and within a specified range defined by min and max.
pub fn gen_range(input: &str, min: u32, max: u32) -> u32 {
    // Hash the input string to create a seed
    // let mut hasher = DefaultHasher::new();
    // input.hash(&mut hasher);
    // let seed = hasher.finish();

    // // Create a seeded random number generator
    // let mut rng = StdRng::seed_from_u64(seed);

    // // Generate a random number within the specified range
    // rng.gen_range(min..max)

    // Hash the input string to create a seed
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let seed = hasher.finish();

    // Create a seeded random number generator
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate a random number within the specified range
    rng.gen_range(min..max)

    // 2
}
