use std::collections::HashSet;

/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};

struct PsiParty {
    name: String,
    intervals: HashSet<Interval>
}

impl PsiParty {
    fn new(name: String) -> Self {
        PsiParty { name, intervals: HashSet::new()}
    }

    fn add_interval(&mut self, bounds: (u8, u8)) {
        self.intervals.insert(Interval { lower: bounds.0, upper: bounds.1 });
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct Interval {
    lower: u8,
    upper: u8,
}

impl Interval {
    fn new(lower: u8, upper: u8) -> Self {
        Interval { lower, upper }
    }

    fn encrypt_interval(&self, encryption_key: &ClientKey) -> (FheUint8,FheUint8) {
        let encrypted_lower = FheUint8::encrypt(self.lower, encryption_key); 
        let encrypted_upper = FheUint8::encrypt(self.upper, encryption_key); 

        return (encrypted_lower, encrypted_upper);
    }
}

//? This example is baased on the fact that the intervals are ordered 
//? in ASC order, with regards to the lower bound

//? At the same time, we are assuming non-overlaping intervals

fn main() {
    /* ======================== INITALIZE FHE SCHEMA  ======================== */
    let config = ConfigBuilder::default().build();

    // Create the key_pair
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    /* ======================== CREATE THE INTERVALS ======================== */
    let mut sender = PsiParty::new(String::from("Sender"));
    let mut receiver = PsiParty::new(String::from("Receiver"));
    
    let sender_intervals: [(u8,u8); 3] = [(2,3), (5,6), (12, 15)];
    let receiver_intervals: [(u8,u8);2] = [(2,6), (13,14)];

    // Add the intervals to the correct psi party
    for i in 0..sender_intervals.len() {
        sender.add_interval(sender_intervals[i]);
    }

    for i in 0..receiver_intervals.len() {
        receiver.add_interval(receiver_intervals[i]);
    }

    // Encrypt the intervals
    let encrypted_sender_intervals: Vec<(FheUint8, FheUint8)> = sender.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();
    let encrypted_receiver_intervals: Vec<(FheUint8, FheUint8)> = receiver.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();


}

