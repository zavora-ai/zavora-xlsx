//! Encryption/decryption support for password-protected Excel files.
//!
//! This module is gated behind the `crypto` feature flag. Currently it provides
//! skeleton APIs that return `UnsupportedFormat` errors. Full encryption support
//! (AES-128-CBC Standard, AES-256-CBC Agile) will be implemented when the
//! `aes`, `sha2`, `hmac`, `cbc`, and `pbkdf2` dependencies are added.

/// OLE2 Compound Document magic bytes: `D0 CF 11 E0 A1 B1 1A E1`
pub const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Check whether the given data starts with the OLE2 magic bytes,
/// indicating an encrypted (or legacy binary) Excel file.
pub fn is_ole2(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == OLE2_MAGIC
}

/// Attempt to decrypt an OLE2-encrypted xlsx file using the given password.
///
/// Currently returns `UnsupportedFormat` — full implementation requires the
/// `crypto` feature flag with AES/SHA dependencies.
pub fn decrypt(_data: &[u8], _password: &str) -> crate::Result<Vec<u8>> {
    Err(crate::Error::UnsupportedFormat(
        "File encryption/decryption is not yet implemented. \
         Enable the 'crypto' feature flag when available."
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ole2_magic_detection() {
        // Valid OLE2 header
        let ole2_header: Vec<u8> = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00];
        assert!(is_ole2(&ole2_header));

        // ZIP header (PK)
        let zip_header: Vec<u8> = vec![0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        assert!(!is_ole2(&zip_header));

        // Too short
        assert!(!is_ole2(&[0xD0, 0xCF]));

        // Empty
        assert!(!is_ole2(&[]));
    }

    #[test]
    fn test_decrypt_returns_unsupported() {
        let dummy = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        let result = decrypt(&dummy, "password");
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::Error::UnsupportedFormat(msg) => {
                assert!(msg.contains("encryption"));
            }
            other => panic!("Expected UnsupportedFormat, got: {other:?}"),
        }
    }
}
