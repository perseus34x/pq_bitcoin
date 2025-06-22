use alloy_sol_types::sol;
 
use alloy_sol_types::SolType;

use hashes::{ripemd160, sha256, Hash};
use secp256k1::Message;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        bytes btc_address;     // fixed-length address
        bytes pq_pubkey;       // fixed-length pq public key
    }
}


pub fn public_key_to_address(pubkey: &[u8]) -> Vec<u8> {
    assert_eq!(pubkey.len(), 33, "Public key should be 33 bytes (compressed)");
    // Step 1: SHA-256 hash
    let msg = sha256::Hash::hash(pubkey);
    let msg = Message::from_digest_slice(msg.as_ref()).unwrap();

    // println!("first sha256 {:?}", hex::encode(msg.as_ref()));

    // Step 2: RIPEMD-160 hash
    let ripemd160_hash = ripemd160::Hash::hash(msg.as_ref());
    let msg :  &[u8] = ripemd160_hash.as_ref();

    // println!("ripemd {:?}", hex::encode(msg));

    // Step 3: Add version byte (0x00 for mainnet)
    let mut versioned_payload = vec![0x00];
    versioned_payload.extend(msg);

    // Step 4: Checksum calculation (double SHA-256)
    let msg = sha256::Hash::hash(versioned_payload.as_slice());
    let msg = Message::from_digest_slice(msg.as_ref()).unwrap();
    let msg = sha256::Hash::hash(msg.as_ref());
    let checksum_full = Message::from_digest_slice(msg.as_ref()).unwrap();
    let checksum = &checksum_full[0..4];

    // Step 5: Append checksum
    versioned_payload.extend(checksum);
    versioned_payload

}
 