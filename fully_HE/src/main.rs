use std::collections::HashSet;

/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};

struct PsiParty {
    name: String,
    encryption_key: ClientKey,
    intervals: HashSet<Interval>
}

impl PsiParty {
    fn new(name: String, encryption_key: ClientKey) -> Self {
        PsiParty { name, intervals: HashSet::new(), encryption_key }
    }

    fn add_interval(&mut self, lower: u8, upper: u8) {
        self.intervals.insert(Interval { lower, upper });
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Interval {
    lower: u8,
    upper: u8,
}

impl Interval {
    fn new(lower: u8, upper: u8) -> Self {
        Interval { lower, upper }
    }
}

fn main() {
    /* ======================== INITALIZE FHE SCHEMA  ======================== */
    let config = ConfigBuilder::default().build();

    // Create the key_pair
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    /* ======================== CREATE THE INTERVALS ======================== */

    let clear_a = 7u8;
    let clear_b= 128u8;

    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);


    let result = a + b;

    let decrypted_result: u8 = result.decrypt(&client_key);

    let clear_result = clear_a + clear_b;

    println!("The HE result is: {:?}", decrypted_result);
    println!("The clear result is: {:?}", clear_result);
}

