/// Validation functions for authentication
use super::config::PasswordRequirements;

/// Validate family name
///
/// Rules:
/// - Must be between 3 and 30 characters
/// - Must start with a letter
/// - Can contain letters (including accented), spaces, hyphens, and apostrophes
/// - No leading or trailing whitespace
pub fn validate_family_name(name: &str) -> Result<(), String> {
    // Trim and check if empty
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Le nom de famille ne peut pas être vide".to_string());
    }

    // Check length
    if trimmed.len() < 3 {
        return Err("Le nom de famille doit contenir au moins 3 caractères".to_string());
    }
    if trimmed.len() > 30 {
        return Err("Le nom de famille ne peut pas dépasser 30 caractères".to_string());
    }

    // Check for leading/trailing whitespace in original string
    if name != trimmed {
        return Err("Le nom de famille ne doit pas commencer ou finir par un espace".to_string());
    }

    // Check first character is a letter
    let first_char = trimmed.chars().next().unwrap();
    if !first_char.is_alphabetic() {
        return Err("Le nom de famille doit commencer par une lettre".to_string());
    }

    // Check all characters are valid (letters, spaces, hyphens, apostrophes)
    for ch in trimmed.chars() {
        if !ch.is_alphabetic() && ch != ' ' && ch != '-' && ch != '\'' {
            return Err(format!(
                "Le nom de famille contient un caractère invalide: '{}'",
                ch
            ));
        }
    }

    // Check for consecutive spaces
    if trimmed.contains("  ") {
        return Err("Le nom de famille ne doit pas contenir plusieurs espaces consécutifs".to_string());
    }

    Ok(())
}

/// Validate password against requirements
pub fn validate_password(password: &str, requirements: &PasswordRequirements) -> Result<(), String> {
    // Check length
    if password.len() < requirements.min_length {
        return Err(format!(
            "Le mot de passe doit contenir au moins {} caractères",
            requirements.min_length
        ));
    }

    // Check for uppercase
    if requirements.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err("Le mot de passe doit contenir au moins une majuscule".to_string());
    }

    // Check for lowercase
    if requirements.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err("Le mot de passe doit contenir au moins une minuscule".to_string());
    }

    // Check for digit
    if requirements.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Le mot de passe doit contenir au moins un chiffre".to_string());
    }

    // Check for special character
    if requirements.require_special {
        let has_special = password
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
        if !has_special {
            return Err("Le mot de passe doit contenir au moins un caractère spécial".to_string());
        }
    }

    // Check for whitespace (generally not allowed)
    if password.contains(char::is_whitespace) {
        return Err("Le mot de passe ne doit pas contenir d'espaces".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_family_name_valid() {
        assert!(validate_family_name("Stark").is_ok());
        assert!(validate_family_name("House Stark").is_ok());
        assert!(validate_family_name("O'Brien").is_ok());
        assert!(validate_family_name("De La Cruz").is_ok());
        assert!(validate_family_name("Château-Renard").is_ok());
        assert!(validate_family_name("D'Artagnan").is_ok());
    }

    #[test]
    fn test_validate_family_name_too_short() {
        assert!(validate_family_name("Ab").is_err());
        assert!(validate_family_name("X").is_err());
    }

    #[test]
    fn test_validate_family_name_too_long() {
        assert!(validate_family_name("ThisFamilyNameIsWayTooLongAndShouldBeRejected").is_err());
    }

    #[test]
    fn test_validate_family_name_invalid_start() {
        assert!(validate_family_name("123Invalid").is_err());
        assert!(validate_family_name("-Invalid").is_err());
        assert!(validate_family_name("'Invalid").is_err());
    }

    #[test]
    fn test_validate_family_name_invalid_characters() {
        assert!(validate_family_name("Invalid@Name").is_err());
        assert!(validate_family_name("Invalid123").is_err());
        assert!(validate_family_name("Invalid_Name").is_err());
    }

    #[test]
    fn test_validate_family_name_whitespace() {
        assert!(validate_family_name(" Stark").is_err());
        assert!(validate_family_name("Stark ").is_err());
        assert!(validate_family_name("House  Stark").is_err()); // Double space
    }

    #[test]
    fn test_validate_family_name_empty() {
        assert!(validate_family_name("").is_err());
        assert!(validate_family_name("   ").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("ValidPass123", &reqs).is_ok());
        assert!(validate_password("MyP@ssw0rd", &reqs).is_ok());
        assert!(validate_password("Test1234", &reqs).is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("Short1", &reqs).is_err());
        assert!(validate_password("Pass1", &reqs).is_err());
    }

    #[test]
    fn test_validate_password_missing_uppercase() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("alllowercase123", &reqs).is_err());
    }

    #[test]
    fn test_validate_password_missing_lowercase() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("ALLUPPERCASE123", &reqs).is_err());
    }

    #[test]
    fn test_validate_password_missing_digit() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("NoDigitsHere", &reqs).is_err());
    }

    #[test]
    fn test_validate_password_with_whitespace() {
        let reqs = PasswordRequirements::default();
        assert!(validate_password("Pass word123", &reqs).is_err());
        assert!(validate_password("Pass 123", &reqs).is_err());
    }

    #[test]
    fn test_validate_password_custom_requirements() {
        let reqs = PasswordRequirements::new(12, true, true, true, true);
        assert!(validate_password("Short1", &reqs).is_err()); // Too short
        assert!(validate_password("LongButNoSpecial123", &reqs).is_err()); // No special char
        assert!(validate_password("LongWith@Special123", &reqs).is_ok()); // All requirements met
    }
}
