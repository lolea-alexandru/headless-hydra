use rand::{Rng, SeedableRng, rngs::StdRng};
use hmac::{Hmac, Mac, digest::typenum::int};
use sha2::Sha256;
use std::{fs::File, time::Instant, io::Write};
use std::fs;
use serde::{Serialize, Deserialize};
type HmacSha256 = Hmac<Sha256>;
use std::path::Path;

const M: u32 = 3;

/// Generates a 128-bit secret key
fn ore_setup() -> u128 {
    let mut rng = StdRng::from_os_rng();

    return rng.random();
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Copy)]
struct Interval {
    lower: u32,
    upper: u32,
}

/// Encrypts the 32-bit plaintext into an equally long cyphertext
fn ore_encrypt(ore_key: u128, plaintext: u32) -> [u8; 32] {
    let mut cyphertext = [0u8; 32];

    // Go through every bit of the plaintext
    for i in 0..32 {
        // What ends up as a parameter, alongside the key, to the PRF (HMAC_SHA256)
        let mut plaintext_fraction: u32 = 0;
        let i_th_bit;

        // ========= COMPUTE BIT IN POSITION i ========= //
        let one_in_first_pos: u32 = 1 << 31;
        i_th_bit = ((plaintext & (one_in_first_pos >> i)) != 0) as u8;

        // ========= COMPUTE CRF FEED ========= //
        for j in 0..i {
            let start_bit = one_in_first_pos >> j;
            let b_j = plaintext & start_bit;
            
            // If bit at position j is different than 0, then add two to that power => mark 1 in position j
            if b_j != 0 {
                plaintext_fraction += (2 as u32).pow(31-j)
            }
        }

        let prf_result = prf(ore_key, i as u8,plaintext_fraction);

        cyphertext[i as usize] = (prf_result + i_th_bit) % M as u8;
        
        // We are using 32-bit values. In order to get the ith bit 
    }

    return cyphertext;
}

// Returns:
// 0
fn ore_compare(ct_1: [u8; 32], ct_2: [u8; 32]) -> i8 {
    for i in 0..32 {
        if ct_1[i] > ct_2[i] {return 1;}
        if ct_1[i] < ct_2[i] {return -1;}
    }
    return 0;
}

fn prf(ore_key: u128, corresponding_bit: u8,plaintext_fraction: u32) -> u8 {
    // Convert the key into byte array
    let ore_key_bytes = ore_key.to_be_bytes();

    // Convert the input into byte arrays (pieces)
    let corresponding_bit_bytes = corresponding_bit.to_be_bytes();
    let separator = b"|";
    let plaintext_fraction_bytes = plaintext_fraction.to_be_bytes();

    // Create the MAC = Message Authentication Code
    let mut mac = HmacSha256::new_from_slice(&ore_key_bytes).expect("Something went wrong at creating the MAC");


    // Add the input to the MAC
    mac.update(&corresponding_bit_bytes);
    mac.update(separator);
    mac.update(&plaintext_fraction_bytes);

    // Compute the result (take only the first 32 bits)
    let result = mac.finalize().into_bytes();
    let first_4_bytes: [u8; 4] = result[0..4].try_into().expect("Something went wrong while retrieving the first 4 bits");
    let hash_32_bits = u32::from_be_bytes(first_4_bytes);

    // compute the modulus
    let remainder = hash_32_bits % M;

    return remainder as u8;
}

fn compute_intersection_ore(ore_key: u128, alice: (u32, u32), bob: (u32, u32)) -> ([u8; 32], [u8; 32]) {
    // Compute encrypted interval beginnings
    let alice_begining = ore_encrypt(ore_key, alice.0);
    let bob_begining = ore_encrypt(ore_key, bob.0);

    // Compute encryptes inverval ends
    let alice_end = ore_encrypt(ore_key, alice.1);
    let bob_end = ore_encrypt(ore_key, bob.1);

    // Decide which beginning to choose
    let mut beginning = alice_begining;
    if ore_compare(alice_begining, bob_begining) == -1 {
        beginning = bob_begining;
    }

    // Decide which end to choose
    let mut end = alice_end;
    if ore_compare(alice_end, bob_end) == 1 {
        end = bob_end;
    }

    return (beginning, end);
}

// CSV writing 
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

fn run_ore(interval_1: &[Interval], interval_2: &[Interval]) -> u64 {

    let startTime = Instant::now();
    
    // This first version of the algorithm will follow these steps: 
    // 1. Generate ORE secret key
    // 2. Encrypt intervals of Alice and Bob using the secret key
    // 3. Compute the interval intersection in O(a*b) complexity, where a = |Alice| and b = |Bob|
    // 4. Decrypt the results
    
    // Assumptions:
    // 1. The intervals of each party are not overlapping
    
    /* =========================== STEP 1 =========================== */
    let ore_key = ore_setup();
   
    println!("The size of the first set is: {:?}", interval_1.len());
    println!("The size of the second set is: {:?}", interval_2.len());
    
    // Go through Alice and Bob's intervals
    let mut intersection: Vec<([u8; 32], [u8; 32])> = Vec::new();
    
    for alice in 0..interval_1.len() {
        for bob in 0..interval_2.len() {
            // Check if Bob is too far
            if interval_2[bob].lower > interval_1[alice].upper {
                break;
            }
            
            // Check if Alice is too far
            if interval_1[alice].lower > interval_2[bob].upper {
                break;
            }
            
            // Determine intersection
            intersection.push(compute_intersection_ore(ore_key, (interval_1[alice].lower, interval_1[alice].upper), (interval_2[bob].lower, interval_2[bob].upper)));
        }
    }

    let json_encrypted_intersections = serde_json::to_string(&intersection).unwrap();
    
    let mut results_file = File::create("src/intersection_result.json").unwrap();
    results_file.write_all(json_encrypted_intersections.as_bytes()).unwrap();
    
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
                let elapsed_ms = run_ore(&interval_1, &interval_2);
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

