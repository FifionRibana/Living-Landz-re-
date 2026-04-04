/// Shared netcode configuration between server and auth endpoint.
/// In production, PRIVATE_KEY should come from a secret manager / env var.
/// For development, a fixed key is acceptable.

/// Protocol ID — must match between server NetcodeConfig and ConnectToken generation.
/// Change this when making breaking protocol changes.
pub const PROTOCOL_ID: u64 = 0x4C495645_4C414E44; // "LIVELAND" in ASCII hex

/// Private key for netcode token signing/verification.
/// 32 bytes. Server and auth endpoint must use the same key.
///
/// TODO(#194): In production, load from NETCODE_PRIVATE_KEY env var.
/// For now, uses a fixed dev key.
pub fn private_key() -> [u8; 32] {
    // Default dev key — NOT secure, for local testing only
    [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
        0x1d, 0x1e, 0x1f, 0x20,
    ]
}
