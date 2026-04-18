//! Cryptographic utilities for dWallet protocol
//!
//! Provides cryptographic operations used across the blockchain.

use sp_core::{H256, sr25519};
use sp_runtime::traits::Verify;
use codec::Encode;

/// Hash data using Blake2-256
pub fn blake2_256_hash(data: &[u8]) -> H256 {
    sp_core::blake2_256(data).into()
}

/// Hash encoded data
pub fn hash_encoded<T: Encode>(value: &T) -> H256 {
    blake2_256_hash(&value.encode())
}

/// Verify sr25519 signature
pub fn verify_signature(
    signature: &sr25519::Signature,
    message: &[u8],
    public_key: &sr25519::Public,
) -> bool {
    signature.verify(message, public_key)
}

/// Verify signature on encoded data
pub fn verify_signature_encoded<T: Encode>(
    signature: &sr25519::Signature,
    value: &T,
    public_key: &sr25519::Public,
) -> bool {
    verify_signature(signature, &value.encode(), public_key)
}

/// Generate a deterministic hash from multiple inputs
pub fn multi_hash<H: Encode>(items: &[H]) -> H256 {
    let mut data = Vec::new();
    for item in items {
        data.extend_from_slice(&item.encode());
    }
    blake2_256_hash(&data)
}

/// Calculate Merkle root from leaves
pub fn calculate_merkle_root(leaves: &[H256]) -> H256 {
    if leaves.is_empty() {
        return H256::default();
    }
    
    if leaves.len() == 1 {
        return leaves[0];
    }
    
    let mut current_level = leaves.to_vec();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(2) {
            let combined = if chunk.len() == 2 {
                hash_encoded(&(&chunk[0], &chunk[1]))
            } else {
                // Odd number of nodes, duplicate the last one
                hash_encoded(&(&chunk[0], &chunk[0]))
            };
            next_level.push(combined);
        }
        
        current_level = next_level;
    }
    
    current_level[0]
}

/// Generate Merkle proof for a leaf
/// Returns (proof_hashes, leaf_index)
pub fn generate_merkle_proof(leaves: &[H256], leaf_index: usize) -> Option<(Vec<H256>, usize)> {
    if leaf_index >= leaves.len() {
        return None;
    }
    
    let mut proof = Vec::new();
    let mut current_level = leaves.to_vec();
    let mut current_index = leaf_index;
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(2) {
            let combined = if chunk.len() == 2 {
                hash_encoded(&(&chunk[0], &chunk[1]))
            } else {
                hash_encoded(&(&chunk[0], &chunk[0]))
            };
            next_level.push(combined);
        }
        
        // Add sibling to proof
        let sibling_index = if current_index % 2 == 0 {
            current_index + 1
        } else {
            current_index - 1
        };
        
        if sibling_index < current_level.len() {
            proof.push(current_level[sibling_index]);
        } else {
            // Duplicate last node if odd
            proof.push(current_level[current_index]);
        }
        
        current_level = next_level;
        current_index /= 2;
    }
    
    Some((proof, leaf_index))
}

/// Verify Merkle proof
pub fn verify_merkle_proof(
    leaf: H256,
    proof: &[H256],
    leaf_index: usize,
    root: H256,
) -> bool {
    let mut current_hash = leaf;
    let mut index = leaf_index;
    
    for &proof_hash in proof {
        if index % 2 == 0 {
            current_hash = hash_encoded(&(&current_hash, &proof_hash));
        } else {
            current_hash = hash_encoded(&(&proof_hash, &current_hash));
        }
        index /= 2;
    }
    
    current_hash == root
}

/// Create a commitment: hash(data + nonce)
pub fn create_commitment<T: Encode>(data: &T, nonce: &[u8]) -> H256 {
    let mut data_bytes = data.encode();
    data_bytes.extend_from_slice(nonce);
    blake2_256_hash(&data_bytes)
}

/// Verify commitment
pub fn verify_commitment<T: Encode>(
    commitment: H256,
    data: &T,
    nonce: &[u8],
) -> bool {
    create_commitment(data, nonce) == commitment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2_256_hash() {
        let data = b"test data";
        let hash = blake2_256_hash(data);
        assert_eq!(hash.as_ref().len(), 32);
    }

    #[test]
    fn test_hash_encoded() {
        let value: u64 = 42;
        let hash = hash_encoded(&value);
        assert_eq!(hash.as_ref().len(), 32);
    }

    #[test]
    fn test_multi_hash() {
        let items: Vec<u64> = vec![1, 2, 3];
        let hash = multi_hash(&items);
        assert_eq!(hash.as_ref().len(), 32);
    }

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaf = H256::from([1u8; 32]);
        let root = calculate_merkle_root(&[leaf]);
        assert_eq!(root, leaf);
    }

    #[test]
    fn test_merkle_root_multiple_leaves() {
        let leaf1 = H256::from([1u8; 32]);
        let leaf2 = H256::from([2u8; 32]);
        let root = calculate_merkle_root(&[leaf1, leaf2]);
        assert_eq!(root.as_ref().len(), 32);
    }

    #[test]
    fn test_merkle_proof_verification() {
        let leaf1 = H256::from([1u8; 32]);
        let leaf2 = H256::from([2u8; 32]);
        let leaves = vec![leaf1, leaf2];
        let root = calculate_merkle_root(&leaves);
        
        let (proof, index) = generate_merkle_proof(&leaves, 0).unwrap();
        assert!(verify_merkle_proof(leaf1, &proof, index, root));
    }

    #[test]
    fn test_commitment() {
        let data: u64 = 12345;
        let nonce = b"secret_nonce";
        let commitment = create_commitment(&data, nonce);
        
        assert!(verify_commitment(commitment, &data, nonce));
        assert!(!verify_commitment(commitment, &data, b"wrong_nonce"));
        assert!(!verify_commitment(commitment, &99999u64, nonce));
    }
}
