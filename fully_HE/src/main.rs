/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8, FheUint16};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::time::Instant;

struct PsiParty {
    name: String,
    intervals: Vec<Interval>
}

impl PsiParty {
    fn new(name: String) -> Self {
        PsiParty { name, intervals: Vec::new()}
    }

    fn add_interval(&mut self, interval: Interval) {
        self.intervals.push(interval);
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Copy)]
struct Interval {
    lower: u16,
    upper: u16,
}

impl Interval {
    fn new(lower: u16, upper: u16) -> Self {
        Interval { lower, upper }
    }

    fn encrypt_interval(&self, encryption_key: &ClientKey) -> (FheUint16,FheUint16) {
        let encrypted_lower = FheUint16::encrypt(self.lower, encryption_key); 
        let encrypted_upper = FheUint16::encrypt(self.upper, encryption_key); 

        return (encrypted_lower, encrypted_upper);
    }
}

//? At the same time, we are assuming non-overlaping intervals

fn compare_encrypted_intervals(a: &(FheUint16,FheUint16), b: &(FheUint16,FheUint16), keys: &ClientKey) -> Option<(FheUint16, FheUint16)> {
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
    let startTime = Instant::now();

    println!("{}", std::env::current_dir().unwrap().display());
    /* ======================== INITALIZE FHE SCHEMA  ======================== */
    let config = ConfigBuilder::default().build();

    // Create the key_pair
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    /* ======================== CREATE THE INTERVALS ======================== */
    let mut sender = PsiParty::new(String::from("Sender"));
    let mut receiver = PsiParty::new(String::from("Receiver"));
    
    // Retrieve the intervals from the JSON file
    
    let data = fs::read_to_string("src/intervals.json").expect("Should be able to open 'intervals.json' file");
    let sender_intervals_json: Vec<Interval> = serde_json::from_str(&data).unwrap();
    let receiver_intervals_json: Vec<Interval> = Vec::from([sender_intervals_json[0]]);

    // TODO: remove -> from previous implementation
    // let sender_intervals: [(u16,u16); 3] = [(2,3), (5,8) ,(12, 15)];
    // let receiver_intervals: [(u16,u16); 2] = [(2,6), (13,14)];

    // Add the intervals to the correct psi party
    for i in 0..sender_intervals_json.len() {
        sender.add_interval(sender_intervals_json[i]);
    }

    for i in 0..receiver_intervals_json.len() {
        receiver.add_interval(receiver_intervals_json[i]);
    }

        
    // Encrypt the intervals
    let encrypted_sender_intervals: Vec<(FheUint16, FheUint16)> = sender.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();
    let encrypted_receiver_intervals: Vec<(FheUint16, FheUint16)> = receiver.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();

    
    let mut encrypted_intersections: Vec<(u16, u16)> = Vec::new(); 
    
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

    let duration = startTime.elapsed();

    println!("The encrypted is: {:?}", encrypted_intersections);
    println!("The intersection was computed in: {:?}", duration);
}

