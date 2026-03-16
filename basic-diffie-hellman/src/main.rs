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

fn map_to_group(input: &u32) -> u64 {
    let hash = Sha256::digest(input.to_be_bytes());
    
    // Retrieve the first eight bytes of the hash 
    // We retrieve only 8 because this toy model uses u64, which means 64b = 8B
    let first_eight_bytes: [u8; 8] = hash[0..8].try_into().expect("Something went wrong");
    let number = u64::from_be_bytes(first_eight_bytes);

    let mapped_val = number % P;

    if mapped_val == 0  {
        return 1;
    }
    return mapped_val;
}

fn modulus_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {return 0;}

    let mut result = 1;
    base = base % modulus;
    
    // Algorithm taken from https://en.wikipedia.org/wiki/Modular_exponentiation
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }

        base = (base * base) % modulus;
        exp = exp / 2;
    }   

    return result;
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
const P: u64 = 65519;


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
                6,
                alice_elements
            );
        
            let mut bob = PsiParty::new(
                String::from("Bob"),
                3,
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
        }
    }

    let duration = startTime.elapsed();

    println!("The intersection was computed in: {:?}", duration);

    // We assume from now on that Alice is the one who needs to find the intersection
    // In order to do this, she needs to keep a "log" of the elements with their original values before being blinded   
    // However, this will not work due to the fact that a HashSet introduces randomness

}
