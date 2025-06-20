

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use pq_bitcoin_lib::{private_polinom, PublicValuesStruct};


pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let x = sp1_zkvm::io::read::<u32>();
    let a = sp1_zkvm::io::read::<u32>();
    let b = sp1_zkvm::io::read::<u32>();

    //y = x^3 + ax + b, s.t 'x' and 'a' are public, but b is private
    let y = private_polinom(x,a,b);

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { x,a,y });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
