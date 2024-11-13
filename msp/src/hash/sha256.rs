use crypto::sha2::Sha256;
use crypto::digest::Digest;
use crate::bytes_to_hex_str;

/// Hashes the given bytes with SHA-256
pub fn hash_sha256(bytes: &[u8]) -> Vec<u8> {
    let mut sha = Sha256::new();
    sha.input(bytes);
    let mut bytes = Vec::new();
    let hash: &mut [u8] = &mut [0; 32];
    sha.result(hash);
    bytes.extend(hash.iter());
    Vec::from(bytes_to_hex_str(bytes.as_slice()))
}

/// Verifies that the SHA-256 hash of the given content matches the given hash
pub fn verify_sha256(content: &[u8], content_hash: &[u8]) -> bool {
    let computed_sha256 = hash_sha256(&content);
    if computed_sha256.as_slice() != content_hash {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nodes must be able to verify SHA-256 hashes to properly validate consensus messages from
        /// other peers, especially those that are used in consensus seals. This allows the network to
        /// verify the origin of the messages and prevents a malicious node from forging messages.
        ///
        /// This test will verify that the `verify_sha512` function properly verifies a SHA-256 hash.
    #[test]
    fn test_sha256_verification() {
        let bytes = b"abc";
        let correct_hash = [
            186, 120, 22, 191, 143, 1, 207, 234, 65, 65, 64, 222, 93, 174, 34, 35, 176, 3, 97, 163, 150, 23, 122, 156, 180, 16, 255, 97, 242, 0, 21, 173,
        ];
        let incorrect_hash = [
            186, 121, 22, 191, 143, 1, 207, 234, 65, 65, 64, 222, 93, 174, 34, 35, 176, 3, 97, 163, 150, 23, 122, 156, 180, 16, 255, 97, 242, 0, 21, 173,
        ];

        assert_eq!(verify_sha256(bytes, &Vec::from(bytes_to_hex_str(&correct_hash))), true);
        assert_eq!(verify_sha256(bytes, &incorrect_hash),false);
    }
}
