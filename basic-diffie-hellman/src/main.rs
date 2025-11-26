use std::collections::HashSet;

struct PsiParty {
    name: String,
    secret_key: u64,
    elements: HashSet<String> 
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 23;

fn main() {
    let alice_elements = ["Arthas", "Illidan", "Malfurion"];
    let bob_elements = ["Arthas", "Thrall", "Vol'jin"];

    let mut alice   = PsiParty {
        name: String::from("Alice"),
        secret_key: 123,
        elements: HashSet::from(alice_elements.map(|str| String::from(str)))
    };

    let mut bob = PsiParty {
        name: String::from("Bob"),
        secret_key: 123,
        elements: HashSet::from(bob_elements.map(|str| String::from(str)))
    };

    println!("Welcome to the PSI demo build on top of Diffie-Hellman");
    println!("The name of the 1st party is: {}", alice.name);
    println!("The name of the 2nd party is: {}", bob.name);
}
