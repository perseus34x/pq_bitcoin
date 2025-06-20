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

use alloy_sol_types::SolType;
use clap::Parser;
use pq_bitcoin_lib::PublicValuesStruct;
use sp1_sdk::{include_elf, HashableKey, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const PROGRAM_ELF: &[u8] = include_elf!("pq_bitcoin-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long, default_value = "7")]
    x: u32,

    #[arg(long, default_value = "3")]
    a: u32,

    #[arg(long, default_value = "8")]
    b: u32,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&args.x);
    println!("x: {}", args.x);

    stdin.write(&args.a);
    println!("a: {}", args.a);

    stdin.write(&args.b);
    println!("b: {}", args.b);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(PROGRAM_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice()).unwrap();
        let PublicValuesStruct { x,a,y } = decoded;
        println!("public values:");
        println!("x: {}", x);
        println!("a: {}", a);
        println!("y: {}", y);

        let expected_y = pq_bitcoin_lib::private_polinom(x,a,args.b);
        assert_eq!(y, expected_y);
        println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(PROGRAM_ELF);
        // println!("pk key {}", pk.pk.);
        println!("vk key {}", vk.bytes32());

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
