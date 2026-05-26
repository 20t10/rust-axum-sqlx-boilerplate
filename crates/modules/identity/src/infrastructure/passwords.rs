pub fn hash_password(raw: &str) -> String {
    format!("hashed::{raw}")
}
