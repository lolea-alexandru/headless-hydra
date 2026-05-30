/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8, FheUint16};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
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

fn compare_encrypted_intervals(a: &(FheUint16,FheUint16), b: &(FheUint16,FheUint16)) -> (FheUint16, FheUint16) {
    //* Check if the two intervals intersect at all
    let start_A_lower_B_start = a.0.le(&b.1);
    let end_A_greater_B_start = a.1.ge(&b.0);

    let both_are_true = start_A_lower_B_start & end_A_greater_B_start;

    //* Compute intersection
    let lower_bound = a.0.max(&b.0);
    let upper_bound = a.1.min(&b.1);

    return (lower_bound, upper_bound);
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
    
    let first_interval_file: String = fs::read_to_string("src/intervals_1.json").expect("Should be able to open 'intervals_1.json' file");
    let secon_interval_file: String = fs::read_to_string("src/intervals_2.json").expect("Should be able to open 'intervals_2.json' file");
    let sender_intervals_json: Vec<Interval> = serde_json::from_str(&first_interval_file).unwrap();
    let receiver_intervals_json: Vec<Interval> = serde_json::from_str(&secon_interval_file).unwrap();

    println!("The size of the first set is: {:?}", sender_intervals_json.len());
    println!("The size of the second set is: {:?}", receiver_intervals_json.len());

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

    
    let mut intersections: Vec<(u16, u16)> = Vec::new(); 
    
    // Go through intervals 
    for i in 0..encrypted_sender_intervals.len() {
        for j in 0..encrypted_receiver_intervals.len() {
            // Compute the intersection of the intervals
            let (left, right) = compare_encrypted_intervals(&encrypted_sender_intervals[i], &encrypted_receiver_intervals[j]);
            
            let decrypted_left = left.decrypt(&client_key);
            let decrypted_right = right.decrypt(&client_key);
            // Pattern match in order to determine if there was an intersection or not
            if decrypted_left <= decrypted_right {
                intersections.push((decrypted_left, decrypted_right));
            }
        }
    }

    let duration = startTime.elapsed();

    // Write the encryption results to a file
    let json_intersections = serde_json::to_string(&intersections).unwrap();

    let mut results_file = File::create("src/intersection_result.json").unwrap();
    results_file.write_all(json_intersections.as_bytes()).unwrap();

    println!("The intersection was computed in: {:?}", duration);
}

