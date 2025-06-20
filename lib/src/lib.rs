use alloy_sol_types::sol;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 x;
        uint32 a;
        uint32 y;
    }
}

/// y = x^3 + ax + b, s.t 'x' and 'a' are public, but b is private
pub fn private_polinom(x: u32, a: u32,  b: u32) -> u32 {
    let mut y = x;
    y = y.wrapping_pow(3);
    y = y.wrapping_add(x.wrapping_mul(a));
    y = y.wrapping_add(b);
    y
}
