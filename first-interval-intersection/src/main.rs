use rand::{Rng, SeedableRng, rngs::StdRng};
use hmac::{Hmac, Mac, digest::typenum::int};
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

        let prf_result = prf(ore_key, i as u8,plaintext_fraction);

        cyphertext[i as usize] = (prf_result + i_th_bit) % M as u8;
        
        // We are using 32-bit values. In order to get the ith bit 
    }

    return cyphertext;
}

// Returns:
// 0
fn ore_compare(ct_1: [u8; 32], ct_2: [u8; 32]) -> i8 {
    for i in 0..32 {
        if ct_1[i] > ct_2[i] {return 1;}
        if ct_1[i] < ct_2[i] {return -1;}
    }
    return 0;
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

fn compute_intersection(ore_key: u128, alice: (u32, u32), bob: (u32, u32)) -> ([u8; 32], [u8; 32]) {
    // Compute encrypted interval beginnings
    let alice_begining = ore_encrypt(ore_key, alice.0);
    let bob_begining = ore_encrypt(ore_key, bob.0);

    // Compute encryptes inverval ends
    let alice_end = ore_encrypt(ore_key, alice.1);
    let bob_end = ore_encrypt(ore_key, bob.1);

    // Decide which beginning to choose
    let mut beginning = alice_begining;
    if ore_compare(alice_begining, bob_begining) == 1 {
        beginning = bob_begining;
    }

    // Decide which end to choose
    let mut end = alice_end;
    if ore_compare(alice_end, bob_end) == -1 {
        end = bob_end;
    }

    return (beginning, end);
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
    let test_2 = ore_encrypt(ore_key, 41);

    // print!("The cyphertext is: {:?}", encrypted_test);

    let alice_intervals: [(u32, u32); 3] = [(1, 5), (6, 7), (9, 12)];
    let bob_intervals: [(u32, u32); 1] = [(2, 4)];

    // Go through Alice and Bob's intervals
    let mut intersection: Vec<([u8; 32], [u8; 32])> = Vec::new();
    
    let mut current_bob = 0;
    for alice in 0..3 {
        for bob in current_bob..1 {
            // Check if Bob is too far
            if bob_intervals[bob].0 > alice_intervals[alice].1 {
                current_bob = bob; 
                break;
            }
            
            // Check if Alice is too far
            if alice_intervals[alice].0 > bob_intervals[bob].1 {
                break;
            }
            
            // Determine intersection
            intersection.push(compute_intersection(ore_key, alice_intervals[alice], bob_intervals[bob]));
        }
    }

    println!("The intersection is: {:?}", intersection);
}
