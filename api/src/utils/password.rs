use argon2::Argon2;
use argon2::password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/// Hashes the provided password using Argon2 and a randomly generated salt.
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate();
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

#[allow(dead_code)]
/// Verifies password against its Argon2 hash.
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_passwords() {
        let plain = "super-secret";
        let hash = hash_password(plain).expect("hashing should succeed");

        assert_ne!(plain, hash, "hash should not equal the input");
        let matches = verify_password(plain, &hash).expect("verification should succeed");
        assert!(matches, "expected verification to return true");
    }

    #[test]
    fn rejects_wrong_password() {
        let hash = hash_password("correct horse battery").expect("hashing should succeed");
        let matches =
            verify_password("wrong password", &hash).expect("verification should succeed");
        assert!(!matches, "verification must fail for mismatched password");
    }
}
