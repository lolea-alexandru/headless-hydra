use std::collections::HashSet;
use sha2::{Digest, Sha256};

struct PsiParty {
    name: String,
    secret_key: u64,
    elements: HashSet<String> 
}

fn map_to_group(input: &str) -> u64 {
    let hash = Sha256::digest(input.as_bytes());
    
    // Retrieve the first eight bytes of the hash 
    // We retrieve only 8 because this toy model uses u64, which means 64b = 8B
    let first_eight_bytes: [u8; 8] = hash[0..8].try_into().expect("Something went wrong");
    let number = u64::from_be_bytes(first_eight_bytes);

    let mut mapped_val = number % P;

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
    fn new(name: String, secret_key: u64, elements: HashSet<String>) -> Self {
        PsiParty {
            name,
            secret_key,
            elements
        }
    }

    fn process_elements(&self) -> Vec<u64> {
        self.elements.iter().map(|el| {
            // The primitive root
            let g = map_to_group(el);
            
            let g_exp_secret_key = modulus_pow(g, self.secret_key, P);
            return g_exp_secret_key;
        }).collect()
    }

    fn process_peer_elements(&self, peer_blinded_elements: &[u64]) -> Vec<u64> {
        peer_blinded_elements.iter().map(|&el| {
            return modulus_pow(el, self.secret_key, P)
        }).collect()
    }
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 65519;


fn main() {
    let alice_elements = ["x", "y", "z"];
    let bob_elements = ["x_1", "y", "z"];

    let mut alice   = PsiParty::new(
        String::from("Alice"),
        6,
        HashSet::from(alice_elements.map(|str| String::from(str)))
    );

    let mut bob = PsiParty::new(
        String::from("Bob"),
        3,
        HashSet::from(bob_elements.map(|str| String::from(str)))
    );

    println!("Welcome to the PSI demo build on top of Diffie-Hellman");
        
    let alice_blinded = alice.process_elements();
    let bob_blinded = bob.process_elements();

    let alice_final = alice.process_peer_elements(&bob_blinded);
    let bob_final = bob.process_peer_elements(&alice_blinded); 

    let alice_lookup: Vec<u64> = alice_final.into_iter().collect();

    // let intersection: Vec<u64> = bob_final.into_iter().filter(|el| alice_lookup.contains(el)).collect();

    // We assume from now on that Alice is the one who needs to find the intersection
    // In order to do this, she needs to keep a "log" of the elements with their original values before being blinded   

    println!("Alice blinded {:?}", alice_blinded);
    println!("Bob final {:?}", bob_final);
}
