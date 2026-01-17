use rand::Rng;

/// Generate a random idempotency key.
pub fn generate_random_idempotency_key() -> String {
    let mut rng = rand::thread_rng();
    let part1: String = (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();
    let part2: String = (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();
    format!("{}{}", part1, part2)
}

/// Deep copy a serde_json::Value.
/// This is equivalent to the TypeScript deepCopy function.
pub fn deep_copy(value: &serde_json::Value) -> serde_json::Value {
    value.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_idempotency_key() {
        let key1 = generate_random_idempotency_key();
        let key2 = generate_random_idempotency_key();
        assert_eq!(key1.len(), 24);
        assert_eq!(key2.len(), 24);
        assert_ne!(key1, key2);
    }
}
