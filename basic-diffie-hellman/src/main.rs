use std::collections::HashSet;

struct PsiParty {
    name: String,
    secret_key: u64,
    elements: HashSet<String> 
}

impl PsiParty {
    fn new(name: String, secret_key: u64, elements: HashSet<String>) -> Self {
        PsiParty {
            name,
            secret_key,
            elements
        }
    }

    fn test_print(&self) {
        println!("The name of the party is: {}", self.name);
    }
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 23;

// Decide hashing algorithm


fn main() {
    let alice_elements = ["Arthas", "Illidan", "Malfurion"];
    let bob_elements = ["Arthas", "Thrall", "Vol'jin"];

    let mut alice   = PsiParty::new(
        String::from("Alice"),
        123,
        HashSet::from(alice_elements.map(|str| String::from(str)))
    );

    let mut bob = PsiParty::new(
        String::from("Bob"),
        123,
        HashSet::from(bob_elements.map(|str| String::from(str)))
    );

    println!("Welcome to the PSI demo build on top of Diffie-Hellman");
    // println!("The name of the 1st party is: {}", alice.name);
    alice.test_print();
    println!("The name of the 2nd party is: {}", bob.name);
}
