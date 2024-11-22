use sha2::{Digest, Sha256};

fn hash_seed(board_id: &str, username: &str, timestamp: u64) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(board_id);
    hasher.update(username);
    hasher.update(timestamp.to_string());
    let result = hasher.finalize();
    u32::from_le_bytes(result[0..4].try_into().unwrap()) // First 4 bytes as u32
}

pub fn rnd_range(board_id: &str, username: &str, timestamp: u64, min: u32, max: u32) -> u32 {
    let seed = hash_seed(board_id, username, timestamp);
    (seed % (max - min)) + min
}
