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
            
            return g;
        }).collect()
    }
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 23;


fn main() {
    let alice_elements = ["Arthas", "Illidan", "Malfurion"];
    let bob_elements = ["Arthas", "Thrall", "Vol'jin"];

    let mut alice   = PsiParty::new(
        String::from("Alice"),
        4,
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

    println!("Alice is sending: {:?}", alice_blinded);    
    println!("Bob is sending: {:?}", bob_blinded);    
}
