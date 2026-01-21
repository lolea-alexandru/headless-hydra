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
    for i in 0..32 {
        // What ends up as a parameter, alongside the key, to the PRF (HMAC_SHA256)
        let mut crf_feed: u32 = 0;
        let i_th_bit;

        // ========= COMPUTE BIT IN POSITION i ========= //
        let one_in_first_pos: u32 = 1 << 31;
        i_th_bit = ((plaintext & (one_in_first_pos >> i)) != 0) as u8;

        cyphertext[i] = (F(ore_key, i as u8,crf_feed) + i_th_bit) % M;
        
        println!("The {:?}'th bit is: {:?}", i + 1, i_th_bit);
        
        // We are using 32-bit values. In order to get the ith bit 
    }

    return cyphertext;
}

fn F(ore_key: u128, corresponding_bit: u8,plaintext_fraction: u32) -> u8 {
    return 0;
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
    // print!("The cyphertext is: {:?}", encrypted_test);
}
