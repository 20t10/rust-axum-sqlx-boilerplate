use crate::domain::value_objects::password_hash::PasswordHash;

pub fn hash_password(raw: &str) -> PasswordHash {
    PasswordHash(format!("hashed::{raw}"))
}

#[cfg(test)]
mod tests {
    use super::hash_password;

    #[test]
    fn hash_password_returns_password_hash_value_object() {
        let hash = hash_password("secret");

        assert_eq!(hash.0, "hashed::secret");
    }
}
