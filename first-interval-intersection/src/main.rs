use rand::{Rng, SeedableRng, rngs::StdRng};

const M: u8 = 3;

/// Generates a 128-bit secret key
fn ore_setup() -> u128 {
    let mut rng = StdRng::from_os_rng();

    return rng.random();
}

/// Encrypts the 32-bit plaintext into an equally long cyphertext
fn ore_encrypt(ore_key: u128, plaintext: u32) -> [u8; 32] {
    let mut cyphertext = [0u8; 32];

    // Go through every bit of the plaintext


    return cyphertext;
}

/// Blinds the plaintext using AES with the ore_key as its key
fn secure_prf(ore_key: u128, plaintext_tuple: (u8, Vec<u8>)) {

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

    print!("The ORE secret key is: {:?}", encrypted_test);
}
