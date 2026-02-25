/// Password hashing and verification using Argon2id
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using Argon2id with default parameters
///
/// Parameters:
/// - Memory: 19 MiB
/// - Iterations: 2
/// - Parallelism: 1
/// - Hash length: 32 bytes
///
/// Returns PHC-formatted string containing algorithm, parameters, salt, and hash
pub fn hash_password(password: &str) -> Result<String, String> {
    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create Argon2 instance with default parameters (secure defaults)
    let argon2 = Argon2::default();

    // Hash the password
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Verify a password against a hash
///
/// Returns true if the password matches the hash, false otherwise
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    // Parse the PHC-formatted hash string
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;

    // Verify the password
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "TestPassword123";
        let hash = hash_password(password).unwrap();

        // Check that hash is not empty
        assert!(!hash.is_empty());

        // Check that hash starts with Argon2 identifier
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "CorrectPassword123";
        let hash = hash_password(password).unwrap();

        // Verify with correct password
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "CorrectPassword123";
        let hash = hash_password(password).unwrap();

        // Verify with incorrect password
        assert!(!verify_password("WrongPassword123", &hash).unwrap());
    }

    #[test]
    fn test_different_salts() {
        let password = "SamePassword123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes (different salts)
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        let password = "TestPassword123";
        let invalid_hash = "not-a-valid-hash";

        // Should return error for invalid hash format
        assert!(verify_password(password, invalid_hash).is_err());
    }
}
