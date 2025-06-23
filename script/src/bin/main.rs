//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
use hashes::{sha256, Hash};
use pq_bitcoin_lib::{public_key_to_btc_address, public_key_to_eth_address};
use rand::rngs::OsRng;
use rand::TryRngCore;
use secp256k1::{ecdsa, Error, Message, PublicKey, Secp256k1, SecretKey, Signing};
use sp1_sdk::{include_elf, HashableKey, ProverClient, SP1Stdin};
use std::time::{SystemTime, UNIX_EPOCH};
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum ProveType {
    Normal,
    Groth,
    Plonk,
}
pub const PROGRAM_ELF: &[u8] = include_elf!("pq_bitcoin-program");

fn sign<C: Signing>(
    secp: &Secp256k1<C>,
    sec_key: [u8; 32],
) -> Result<(ecdsa::Signature, PublicKey, Vec<u8>), Error> {
    let sec_key = SecretKey::from_slice(&sec_key)?;
    let pub_key = sec_key.public_key(secp);
    let address = public_key_to_eth_address(&pub_key.serialize_uncompressed());

    let msg = sha256::Hash::hash(address.as_slice());
    let msg = Message::from_digest_slice(msg.as_ref())?;

    Ok((secp.sign_ecdsa(&msg, &sec_key), pub_key, address))
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,
    #[clap(long)]
    prove: bool,
    #[clap(long, value_enum, default_value_t = ProveType::Normal)]
    prove_type: ProveType,
}
//RUST_LOG=info cargo run --release -- --prove --prove-type=normal
//RUST_LOG=info cargo run --release -- --prove --prove-type=groth
//RUST_LOG=info cargo run --release -- --prove --prove-type=plonkj
fn main() {
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    println!("Program size is : {}", PROGRAM_ELF.len());
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();

    let secp = Secp256k1::new();
    let mut seckey = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut seckey)
        .expect("cannot fill random bytes");

    let (signature, pub_key, address) = sign(&secp, seckey).unwrap();
    let serialized_pub_key = &pub_key.serialize_uncompressed();
    let serialize_sig = signature.serialize_compact();

    stdin.write(&serialized_pub_key.to_vec());
    stdin.write(&address.to_vec());
    stdin.write(&serialize_sig.to_vec());

    let pq_public_key = b"pq public keys.";
    stdin.write(&pq_public_key.to_vec());

    if args.execute {
        let (_output, report) = client.execute(PROGRAM_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");
        println!("Values are correct!");
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        let (pk, vk) = client.setup(PROGRAM_ELF);
        let system_time = SystemTime::now();
        let start_time = system_time.duration_since(UNIX_EPOCH);
        // println!("pk key {}", pk.pk.);
        println!("vk key {}", vk.bytes32());

        let proof = match args.prove_type {
            ProveType::Groth => client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate Groth16 proof"),

            ProveType::Plonk => client
                .prove(&pk, &stdin)
                .plonk()
                .run()
                .expect("failed to generate Plonk proof"),

            ProveType::Normal => client
                .prove(&pk, &stdin)
                .run()
                .expect("failed to generate normal proof"),
        };
        if matches!(args.prove_type, ProveType::Groth|ProveType::Plonk) {
            println!("Proof size {}", proof.bytes().len());
        }

        println!("Public values size {}", proof.public_values.to_vec().len());
        let system_time = SystemTime::now();
        println!(
            "Elapsed Proving time: {:?}",
            system_time.duration_since(UNIX_EPOCH).unwrap() - start_time.unwrap()
        );
        println!("Successfully generated proof!");
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
