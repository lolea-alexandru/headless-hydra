use std::result;

use num_bigint::BigUint;
use rand::{Rng, SeedableRng, rngs::StdRng};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const M: u32 = 3;

/// Generates a 128-bit secret key
fn ore_setup() -> u128 {
    let mut rng = StdRng::from_os_rng();

    return rng.random();
}

/// Encrypts the 32-bit plaintext into an equally long cyphertext
fn ore_encrypt(ore_key: u128, plaintext: u32) -> [u8; 32] {
    let mut cyphertext = [0u8; 32];

    // Go through every bit of the plaintext
    for i in 0..32 {
        // What ends up as a parameter, alongside the key, to the PRF (HMAC_SHA256)
        let mut plaintext_fraction: u32 = 0;
        let i_th_bit;

        // ========= COMPUTE BIT IN POSITION i ========= //
        let one_in_first_pos: u32 = 1 << 31;
        i_th_bit = ((plaintext & (one_in_first_pos >> i)) != 0) as u8;

        // ========= COMPUTE CRF FEED ========= //
        for j in 0..i {
            let start_bit = one_in_first_pos >> j;
            let b_j = plaintext & start_bit;
            
            // If bit at position j is different than 0, then add two to that power => mark 1 in position j
            if b_j != 0 {
                plaintext_fraction += (2 as u32).pow(31-j)
            }
        }

        println!("The {:?}'th plaintext_fraction is: {:?}", i + 1, format!("{:032b}", plaintext_fraction));
        
        let prf_result = prf(ore_key, i as u8,plaintext_fraction);
        println!("The PRF result is: {:?}", prf_result);

        cyphertext[i as usize] = (prf_result + i_th_bit) % M as u8;
        
        // We are using 32-bit values. In order to get the ith bit 
    }

    return cyphertext;
}

fn prf(ore_key: u128, corresponding_bit: u8,plaintext_fraction: u32) -> u8 {
    // Convert the key into byte array
    let ore_key_bytes = ore_key.to_be_bytes();

    // Convert the input into byte arrays (pieces)
    let corresponding_bit_bytes = corresponding_bit.to_be_bytes();
    let separator = b"|";
    let plaintext_fraction_bytes = plaintext_fraction.to_be_bytes();

    // Create the MAC = Message Authentication Code
    let mut mac = HmacSha256::new_from_slice(&ore_key_bytes).expect("Something went wrong at creating the MAC");


    // Add the input to the MAC
    mac.update(&corresponding_bit_bytes);
    mac.update(separator);
    mac.update(&plaintext_fraction_bytes);

    // Compute the result (take only the first 32 bits)
    let result = mac.finalize().into_bytes();
    let first_4_bytes: [u8; 4] = result[0..4].try_into().expect("Something went wrong while retrieving the first 4 bits");
    let hash_32_bits = u32::from_be_bytes(first_4_bytes);

    // compute the modulus
    let remainder = hash_32_bits % M;

    return remainder as u8;
}

fn main() {
    println!("Welcome to my first implementation of interval intersection built on top of PSI");

    // This first version of the algorithm will follow these steps: 
    // 1. Generate ORE secret key
    // 2. Encrypt intervals of Alice and Bob using the secret key
    // 3. Compute the interval intersection in O(a*b) complexity, where a = |Alice| and b = |Bob|
    // 4. Decrypt the results

    // Assumptions:
    // 1. The intervals of each party are not overlapping

    /* =========================== STEP 1 =========================== */
    let ore_key = ore_setup();

    let plaintext_test = 42;
    let encrypted_test = ore_encrypt(ore_key, plaintext_test);
    let test_2 = ore_encrypt(ore_key, 42);


    // print!("The cyphertext is: {:?}", encrypted_test);
}
