#![no_main]

 use alloy_sol_types::private::Bytes;
use alloy_sol_types::SolType;

use hashes::{sha256, Hash};
use secp256k1::{ecdsa, Error, Message, PublicKey, Secp256k1, Verification};
use pq_bitcoin_lib::{public_key_to_btc_address, public_key_to_eth_address, PublicBTCValues};
sp1_zkvm::entrypoint!(main);


fn verify_btc<C: Verification>(
    secp: &Secp256k1<C>,
    msg: &[u8],
    sig: [u8; 64],
    pubkey: &[u8],
) -> Result<bool, Error> {
    let msg = sha256::Hash::hash(msg);
    let msg = Message::from_digest_slice(msg.as_ref())?;
    let sig = ecdsa::Signature::from_compact(&sig)?;
    let pubkey = PublicKey::from_slice(&pubkey)?;
    let result = secp.verify_ecdsa(&msg, &sig, &pubkey).is_ok();
    Ok(result)
}

pub fn main() {
    let secp = Secp256k1::new();
    let pubkey_vec: Vec<u8> = sp1_zkvm::io::read::<Vec<u8>>();
    // Convert Vec<u8> into [u8; 33]
    let pubkey: [u8; 65] = pubkey_vec
        .try_into()
        .expect("Compressed public key must be exactly 64 bytes");
    let eth_address: Vec<u8> = sp1_zkvm::io::read::<Vec<u8>>();
    let sig_serialized: Vec<u8> = sp1_zkvm::io::read::<Vec<u8>>();
    let pq_public_key: Vec<u8> = sp1_zkvm::io::read::<Vec<u8>>();

    assert!(verify_btc(&secp, eth_address.as_slice(), sig_serialized.try_into().unwrap(), &pubkey[..]).unwrap());
    let derived_eth_address = public_key_to_eth_address(&pubkey);
    assert_eq!(derived_eth_address, eth_address);

    // Encode the public values of the program.
   let bytes = PublicBTCValues::abi_encode(&PublicBTCValues {
       btc_address: Bytes::from(eth_address),
       pq_pubkey: Bytes::from(pq_public_key),
   });
    //committing to the public parameters
    sp1_zkvm::io::commit_slice(&bytes);
}


