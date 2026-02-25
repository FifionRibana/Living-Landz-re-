/// Authentication module for shared validation and configuration

pub mod config;
pub mod validation;

pub use config::PasswordRequirements;
pub use validation::{validate_family_name, validate_password};
