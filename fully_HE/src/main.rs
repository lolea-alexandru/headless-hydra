/* ===================== IMPORTS ===================== */
use tfhe::{ClientKey, prelude::*};
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8, FheUint64};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::time::Instant;
use std::path::Path;

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
    lower: u64,
    upper: u64,
}

impl Interval {
    fn new(lower: u64, upper: u64) -> Self {
        Interval { lower, upper }
    }

    fn encrypt_interval(&self, encryption_key: &ClientKey) -> (FheUint64,FheUint64) {
        let encrypted_lower = FheUint64::encrypt(self.lower, encryption_key); 
        let encrypted_upper = FheUint64::encrypt(self.upper, encryption_key); 

        return (encrypted_lower, encrypted_upper);
    }
}

//? At the same time, we are assuming non-overlaping intervals

fn compare_encrypted_intervals(a: &(FheUint64,FheUint64), b: &(FheUint64,FheUint64)) -> (FheUint64, FheUint64) {
    //* Check if the two intervals intersect at all
    let start_A_lower_B_start = a.0.le(&b.1);
    let end_A_greater_B_start = a.1.ge(&b.0);

    let both_are_true = start_A_lower_B_start & end_A_greater_B_start;

    //* Compute intersection
    let lower_bound = a.0.max(&b.0);
    let upper_bound = a.1.min(&b.1);

    return (lower_bound, upper_bound);
}

const SCALE_ROOT: &str = "src/scaleExperiments";
const MODES: [&str; 2] = ["shuffled", "sorted"];
const N_VALUES: [u32; 1] = [10];
const RUNS_PER_CONFIG: usize = 1;

// ---- Helpers for loading config files ----

fn load_intervals(path: &Path) -> Result<Vec<Interval>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Reading {:?}: {}", path, e))?;
    serde_json::from_str(&text)
        .map_err(|e| format!("Parsing {:?}: {}", path, e))
}

fn run_fhe(interval_1: &[Interval], interval_2: &[Interval]) -> u64 {

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
    
    
    println!("The size of the first set is: {:?}", interval_1.len());
    println!("The size of the second set is: {:?}", interval_2.len());
    
    // TODO: remove -> from previous implementation
    // let sender_intervals: [(u64,u64); 3] = [(2,3), (5,8) ,(12, 15)];
    // let receiver_intervals: [(u64,u64); 2] = [(2,6), (13,14)];
    
    // Add the intervals to the correct psi party
    for i in 0..interval_1.len() {
        sender.add_interval(interval_1[i]);
    }
    
    for i in 0..interval_2.len() {
        receiver.add_interval(interval_2[i]);
    }
    
    println!("Add intervals to stuff");
        
    // Encrypt the intervals
    let encrypted_sender_intervals: Vec<(FheUint64, FheUint64)> = sender.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();
    let encrypted_receiver_intervals: Vec<(FheUint64, FheUint64)> = receiver.intervals.iter().map(|interval| interval.encrypt_interval(&client_key)).collect();
    
    
    let mut intersections: Vec<(u64, u64)> = Vec::new(); 
    
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
    
    
    // Write the encryption results to a file
    let json_intersections = serde_json::to_string(&intersections).unwrap();
    
    let mut results_file = File::create("src/intersection_result.json").unwrap();
    results_file.write_all(json_intersections.as_bytes()).unwrap();
    
    return startTime.elapsed().as_millis() as u64;
}

fn main() {
    for mode in MODES.iter() {
        println!("\n=== Mode: {} ===", mode);

        // Create the matrix of runtimes    
        let mut grid: Vec<Vec<u64>> = vec![vec![0; N_VALUES.len()]; RUNS_PER_CONFIG];
        for (col_idx, &n) in N_VALUES.iter().enumerate() {
            let config_dir = Path::new(SCALE_ROOT)
                .join(mode)
                .join(format!("lineitem_n{}", n));
            let intervals_path = config_dir.join("intervals_1.json");

            let interval_1 = match load_intervals(&intervals_path) {
                Ok(v) => vec![v[0]],
                Err(e) => {
                    eprintln!("  [n={}] FAILED to load: {}", n, e);
                    continue;
                }
            };

            if interval_1.is_empty() {
                eprintln!("  [n={}] intervals_1.json is empty, skipping", n);
                continue;
            }

            let interval_2: Vec<Interval> = vec![interval_1[0]];

            // Three runs of the PSI for this config
            print!("  [n={}] runs:", n);
            for run_idx in 0..RUNS_PER_CONFIG {
                let elapsed_ms = run_fhe(&interval_1, &interval_2);
                grid[run_idx][col_idx] = elapsed_ms;
                print!(" {}ms", elapsed_ms);
            }
            println!();
        }

        // Write this mode's CSV: 3 rows, N_VALUES.len() columns, comma-separated.
        let csv_path = format!("scaleExperiments_{}.csv", mode);
        if let Err(e) = write_csv(&csv_path, &grid) {
            eprintln!("  Failed to write {}: {}", csv_path, e);
        } else {
            println!("  Wrote {}", csv_path);
        }
    }
}

fn write_csv(path: &str, grid: &[Vec<u64>]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    for row in grid {
        let line: Vec<String> = row.iter().map(|v| v.to_string()).collect();
        writeln!(file, "{}", line.join(","))?;
    }
    Ok(())
}


