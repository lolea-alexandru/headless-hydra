use std::collections::HashSet;

/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};

struct PsiParty {
    name: String,
    intervals: Vec<Interval>
}

impl PsiParty {
    fn new(name: String) -> Self {
        PsiParty { name, intervals: Vec::new()}
    }

    fn add_interval(&mut self, bounds: (u8, u8)) {
        self.intervals.push(Interval { lower: bounds.0, upper: bounds.1 });
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

//? in ASC order, with regards to the lower bound

//? At the same time, we are assuming non-overlaping intervals

fn compare_encrypted_intervals(a: &(FheUint8,FheUint8), b: &(FheUint8,FheUint8), keys: &ClientKey) -> Option<(FheUint8, FheUint8)> {
    //* Check if the two intervals intersect at all
    let first_high_lower = a.1.lt(&b.0);
    let second_high_lower = b.1.lt(&a.0);
    if first_high_lower.decrypt(keys) || second_high_lower.decrypt(keys) {
       return None; 
    }

    //* Compute intersection
    let lower_bound;
    let upper_bound;

    // Check which of the lower bounds is largest
    let left_low_smaller = a.0.lt(&b.0).decrypt(keys);
    if left_low_smaller {
        lower_bound = b.0.clone();
    } else {
        lower_bound = a.0.clone();
    }

    // Check which of the upper bounds is smallest
    let left_high_larger = a.1.lt(&b.1).decrypt(keys);
    if left_high_larger {
        upper_bound = a.1.clone();
    } else {
        upper_bound = b.1.clone();
    }


    return Some((lower_bound, upper_bound));

}

fn main() {
    /* ======================== INITALIZE FHE SCHEMA  ======================== */
    let config = ConfigBuilder::default().build();

    // Create the key_pair
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    /* ======================== CREATE THE INTERVALS ======================== */
    let mut sender = PsiParty::new(String::from("Sender"));
    let mut receiver = PsiParty::new(String::from("Receiver"));
    
    let sender_intervals: [(u8,u8); 3] = [(2,3), (5,8) ,(12, 15)];
    let receiver_intervals: [(u8,u8); 2] = [(2,6), (13,14)];

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

    
    let mut encrypted_intersections: Vec<(u8, u8)> = Vec::new(); 
    
    // Go through intervals 
    for i in 0..encrypted_sender_intervals.len() {
        for j in 0..encrypted_receiver_intervals.len() {
            // Compute the intersection of the intervals
            let result = compare_encrypted_intervals(&encrypted_sender_intervals[i], &encrypted_receiver_intervals[j], &client_key);
        
            // Pattern match in order to determine if there was an intersection or not
            match result {
                Some((a, b)) => encrypted_intersections.push((a.decrypt(&client_key), b.decrypt(&client_key))),
                None => (),
            }
        }
    }

    println!("The encrypted is: {:?}", encrypted_intersections);
   
}

