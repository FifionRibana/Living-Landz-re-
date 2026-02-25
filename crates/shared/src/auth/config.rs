/// Password requirements configuration

#[derive(Debug, Clone)]
pub struct PasswordRequirements {
    /// Minimum password length
    pub min_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special: bool,
}

impl Default for PasswordRequirements {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false, // Start lenient
        }
    }
}

impl PasswordRequirements {
    /// Create custom password requirements
    pub fn new(
        min_length: usize,
        require_uppercase: bool,
        require_lowercase: bool,
        require_digit: bool,
        require_special: bool,
    ) -> Self {
        Self {
            min_length,
            require_uppercase,
            require_lowercase,
            require_digit,
            require_special,
        }
    }

    /// Get requirements as human-readable string
    pub fn as_description(&self) -> String {
        let mut requirements = vec![format!("Au moins {} caractères", self.min_length)];

        if self.require_uppercase {
            requirements.push("Une majuscule".to_string());
        }
        if self.require_lowercase {
            requirements.push("Une minuscule".to_string());
        }
        if self.require_digit {
            requirements.push("Un chiffre".to_string());
        }
        if self.require_special {
            requirements.push("Un caractère spécial".to_string());
        }

        requirements.join(", ")
    }
}
