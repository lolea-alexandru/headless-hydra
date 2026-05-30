use std::collections::HashMap;
use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use std::{fs::File, time::Instant, io::Write};
use std::fs;

struct PsiParty {
    name: String,
    secret_key: u64,
    elements: HashMap<u32, u64> 
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Copy)]
struct Interval {
    lower: u32,
    upper: u32,
}

/// True iff `g` is a primitive root modulo P.
///
/// g is a primitive root iff, for every distinct prime factor q of (P-1),
///   g^((P-1)/q) mod P != 1.
fn is_primitive_root(g: u64) -> bool {
    // Trivial small-order elements can never be primitive roots.
    if g <= 1 || g == P - 1 {
        return false;
    }
    let phi = P - 1;
    for &q in P_MINUS_1_FACTORS.iter() {
        if modulus_pow(g, phi / q, P) == 1 {
            return false;
        }
    }
    return true;
}

/// Maps an input value deterministically to a primitive root modulo P.
///
/// Hashes (input || counter); if the candidate (mod P) is not a primitive
/// root, it increments the counter and re-hashes until one is found.
///
/// Determinism guarantee: the same `input` always yields the same returned
/// primitive root, so two parties hashing a shared element agree on g.
fn map_to_group(input: &u32) -> u64 {
    let mut counter: u64 = 0;

    loop {
        let mut hasher = Sha256::new();
        hasher.update(input.to_be_bytes());
        hasher.update(counter.to_be_bytes());
        let hash = hasher.finalize();

        // Take the first 8 bytes -> u64. SHA-256 is 32 bytes, so this is safe.
        let first_eight: [u8; 8] = hash[0..8]
            .try_into()
            .expect("SHA-256 digest is always >= 8 bytes");
        let number = u64::from_be_bytes(first_eight);

        let candidate = number % P;

        if is_primitive_root(candidate) {
            return candidate;
        }

        counter += 1;
    }
}


fn modulus_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u64 = 1;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}

impl PsiParty {
    fn new(name: String, secret_key: u64, elements: HashMap<u32, u64>) -> Self {
        PsiParty {
            name,
            secret_key,
            elements
        }
    }

    fn process_elements(&mut self) -> Vec<u64> {
        let mut blinded_values = Vec::new();

        for (key, value) in self.elements.iter_mut() {
            // The primitive root
            let g = map_to_group(&key);

            let g_exp_secret_key = modulus_pow(g, self.secret_key, P);
            
            blinded_values.push(g_exp_secret_key);

            *value = g_exp_secret_key;
        }

        return blinded_values;
    }

    fn process_peer_element(&self, peer_blinded_element: u64) -> u64 {
        return modulus_pow(peer_blinded_element, self.secret_key, P);
    }
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 4294967387;
const AliceSecret: u64 = 6;
const BobSecret: u64 = 3;
// IMPORTANT: if you change P, you MUST recompute these factors.
const P_MINUS_1_FACTORS: [u64; 2] = [2, 2147483693];

fn main() {
    let startTime = Instant::now();

    let alice_interval_file: String = fs::read_to_string("src/intervals_1.json").expect("Should be able to open 'intervals_1.json' file");
    let bob_interval_file: String = fs::read_to_string("src/intervals_2.json").expect("Should be able to open 'intervals_2.json' file");
    let alice_intervals_json: Vec<Interval> = serde_json::from_str(&alice_interval_file).unwrap();
    let bob_intervals_json: Vec<Interval> = serde_json::from_str(&bob_interval_file).unwrap();
    
    println!("The size of the first set is: {:?}", alice_intervals_json.len());
    println!("The size of the second set is: {:?}", bob_intervals_json.len());

    // Go through every interval, expand it and then compute the intersections
    for alice_i in 0..alice_intervals_json.len() {
        for bob_i in 0..bob_intervals_json.len() {
            let alice_current_interval: Interval = alice_intervals_json[alice_i];
            let bob_current_interval: Interval = bob_intervals_json[bob_i];
            
            let mut alice_elements = HashMap::new();
            let mut bob_elements = HashMap::new();
            for al_i in alice_current_interval.lower..=alice_current_interval.upper {
                alice_elements.insert(al_i, 0);
            }
            for bo_i in bob_current_interval.lower..=bob_current_interval.upper {
                bob_elements.insert(bo_i, 0);
            }

            let mut alice   = PsiParty::new(
                String::from("Alice"),
                AliceSecret,
                alice_elements
            );
        
            let mut bob = PsiParty::new(
                String::from("Bob"),
                BobSecret,
                bob_elements
            );
        
            let alice_blinded = alice.process_elements();
            let bob_blinded = bob.process_elements();
        
            // ! Does this put Alice's/Bob's PK at risk?
            for (_key, value) in bob.elements.iter_mut() {
                *value = alice.process_peer_element(*value);
            }
        
            for (_key, value) in alice.elements.iter_mut() {
                *value = bob.process_peer_element(*value);
            }
        
            let alice_lookup: Vec<&u64> = alice.elements.values().collect();
        
            let intersection_values: Vec<u64> = bob.elements.values().filter(|el| alice_lookup.contains(el)).map(|ref_val| *ref_val).collect();
            
            let mut common_keys: Vec<u32> = Vec::new();
        
            for (key, value) in alice.elements {
                if intersection_values.contains(&value) {
                    common_keys.push(key);
                }
            }

            if !common_keys.is_empty() {
                final_intersection.push(Interval {
                    lower: *common_keys.iter().min().unwrap(),
                    upper: *common_keys.iter().max().unwrap(),
                });
            }    
        }    
    }

    let duration = startTime.elapsed();

    println!("The intersection was computed in: {:?}", duration);

    // We assume from now on that Alice is the one who needs to find the intersection
    // In order to do this, she needs to keep a "log" of the elements with their original values before being blinded   
    // However, this will not work due to the fact that a HashSet introduces randomness

}
