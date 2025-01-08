use std::hash::Hash;

pub fn hash_t(t: &impl Hash) -> u32 {
    let mut hasher = crc32fast::Hasher::new();

    t.hash(&mut hasher);

    hasher.finalize()
}

pub fn hash_t_hex(t: &impl Hash) -> String {
    format!("{hash:X}", hash = hash_t(t))
}
