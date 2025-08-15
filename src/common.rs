use std::hash::{DefaultHasher, Hash, Hasher};

pub fn generate_hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
