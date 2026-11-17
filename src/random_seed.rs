use getrandom::getrandom;

pub fn random_seed() -> u64 {
    let mut bytes = [0u8; 8];
    getrandom(&mut bytes).expect("getrandom failed");
    u64::from_le_bytes(bytes)
}
