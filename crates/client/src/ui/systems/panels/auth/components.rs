/// Components for authentication panels
use bevy::prelude::*;

// ==================== Login Panel Components ====================

/// Marker component for the login panel container
#[derive(Component)]
pub struct LoginPanelMarker;

/// Marker for the family name input field in login panel
#[derive(Component)]
pub struct LoginFamilyNameInput;

/// Marker for the password input field in login panel
#[derive(Component)]
pub struct LoginPasswordInput;

/// Marker for the login submit button
#[derive(Component)]
pub struct LoginSubmitButton;

/// Marker for the "Create account" link button
#[derive(Component)]
pub struct LoginToRegisterButton;

/// Marker for the error message text in login panel
#[derive(Component)]
pub struct LoginErrorText;

// ==================== Register Panel Components ====================

/// Marker component for the register panel container
#[derive(Component)]
pub struct RegisterPanelMarker;

/// Marker for the family name input field in register panel
#[derive(Component)]
pub struct RegisterFamilyNameInput;

/// Marker for the password input field in register panel
#[derive(Component)]
pub struct RegisterPasswordInput;

/// Marker for the password confirmation input field in register panel
#[derive(Component)]
pub struct RegisterPasswordConfirmInput;

/// Marker for the register submit button
#[derive(Component)]
pub struct RegisterSubmitButton;

/// Marker for the back button in register panel
#[derive(Component)]
pub struct RegisterBackButton;

/// Marker for the error message text in register panel
#[derive(Component)]
pub struct RegisterErrorText;

/// Marker for the success message text in register panel
#[derive(Component)]
pub struct RegisterSuccessText;

/// Marker for the password requirements text in register panel
#[derive(Component)]
pub struct RegisterPasswordRequirementsText;

// ==================== Dev/Test Buttons ====================

/// Test button: jump to character creation screen
#[derive(Component)]
pub struct TestCharacterCreationButton;

/// Test button: jump to coat of arms screen
#[derive(Component)]
pub struct TestCoatOfArmsButton;
