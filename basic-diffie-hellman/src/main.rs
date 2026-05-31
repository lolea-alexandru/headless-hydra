use std::collections::HashMap;
use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use std::{fs::File, time::Instant, io::Write};
use std::fs;
use std::path::Path;

// We assume from now on that Alice is the one who needs to find the intersection
// In order to do this, she needs to keep a "log" of the elements with their original values before being blinded   
// However, this will not work due to the fact that a HashSet introduces randomness
struct PsiParty {
    name: String,
    secret_key: u64,
    elements: HashMap<u32, u64> 
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Copy)]
struct Interval {
    lower: u32,
    upper: u32,
}

/// True iff `g` is a primitive root modulo P.
///
/// g is a primitive root iff, for every distinct prime factor q of (P-1),
///   g^((P-1)/q) mod P != 1.
fn is_primitive_root(g: u64) -> bool {
    // Trivial small-order elements can never be primitive roots.
    if g <= 1 || g == P - 1 {
        return false;
    }
    let phi = P - 1;
    for &q in P_MINUS_1_FACTORS.iter() {
        if modulus_pow(g, phi / q, P) == 1 {
            return false;
        }
    }
    return true;
}

/// Maps an input value deterministically to a primitive root modulo P.
///
/// Hashes (input || counter); if the candidate (mod P) is not a primitive
/// root, it increments the counter and re-hashes until one is found.
///
/// Determinism guarantee: the same `input` always yields the same returned
/// primitive root, so two parties hashing a shared element agree on g.
fn map_to_group(input: &u32) -> u64 {
    let mut counter: u64 = 0;

    loop {
        let mut hasher = Sha256::new();
        hasher.update(input.to_be_bytes());
        hasher.update(counter.to_be_bytes());
        let hash = hasher.finalize();

        // Take the first 8 bytes -> u64. SHA-256 is 32 bytes, so this is safe.
        let first_eight: [u8; 8] = hash[0..8]
            .try_into()
            .expect("SHA-256 digest is always >= 8 bytes");
        let number = u64::from_be_bytes(first_eight);

        let candidate = number % P;

        if is_primitive_root(candidate) {
            return candidate;
        }

        counter += 1;
    }
}


fn modulus_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u64 = 1;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}

impl PsiParty {
    fn new(name: String, secret_key: u64, elements: HashMap<u32, u64>) -> Self {
        PsiParty {
            name,
            secret_key,
            elements
        }
    }

    fn process_elements(&mut self) -> Vec<u64> {
        let mut blinded_values = Vec::new();

        for (key, value) in self.elements.iter_mut() {
            // The primitive root
            let g = map_to_group(&key);

            let g_exp_secret_key = modulus_pow(g, self.secret_key, P);
            
            blinded_values.push(g_exp_secret_key);

            *value = g_exp_secret_key;
        }

        return blinded_values;
    }

    fn process_peer_element(&self, peer_blinded_element: u64) -> u64 {
        return modulus_pow(peer_blinded_element, self.secret_key, P);
    }
}

// Safe prime chosen for the implementation of this toy model
const P: u64 = 4294967387;
const AliceSecret: u64 = 6;
const BobSecret: u64 = 3;
// IMPORTANT: if you change P, you MUST recompute these factors.
const P_MINUS_1_FACTORS: [u64; 2] = [2, 2147483693];

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

fn run_psi(interval_1: &[Interval], interval_2: &[Interval]) -> u64 {
    let startTime = Instant::now();
     let mut intersection_final: Vec<Interval> = Vec::new();
    // Go through every interval, expand it and then compute the intersections

    for alice_i in 0..interval_1.len() {
        for bob_i in 0..interval_2.len() {
            let alice_current_interval: Interval = interval_1[alice_i];
            let bob_current_interval: Interval = interval_2[bob_i];
            
            let mut alice_elements = HashMap::new();
            let mut bob_elements = HashMap::new();
            for al_i in alice_current_interval.lower..=alice_current_interval.upper {
                alice_elements.insert(al_i, 0);
            }
            for bo_i in bob_current_interval.lower..=bob_current_interval.upper {
                bob_elements.insert(bo_i, 0);
            }

            let mut alice   = PsiParty::new(
                String::from("Alice"),
                AliceSecret,
                alice_elements
            );
        
            let mut bob = PsiParty::new(
                String::from("Bob"),
                BobSecret,
                bob_elements
            );
        
            let alice_blinded = alice.process_elements();
            let bob_blinded = bob.process_elements();
        
            // ! Does this put Alice's/Bob's PK at risk?
            for (_key, value) in bob.elements.iter_mut() {
                *value = alice.process_peer_element(*value);
            }
        
            for (_key, value) in alice.elements.iter_mut() {
                *value = bob.process_peer_element(*value);
            }
        
            let alice_lookup: Vec<&u64> = alice.elements.values().collect();
        
            let intersection_values: Vec<u64> = bob.elements.values().filter(|el| alice_lookup.contains(el)).map(|ref_val| *ref_val).collect();
            
            let mut common_keys: Vec<u32> = Vec::new();
        
            for (key, value) in alice.elements {
                if intersection_values.contains(&value) {
                    common_keys.push(key);
                }
            }

            if !common_keys.is_empty() {
                intersection_final.push(Interval {
                    lower: *common_keys.iter().min().unwrap(),
                    upper: *common_keys.iter().max().unwrap(),
                });
            }    
        }    
    }

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
                let elapsed_ms = run_psi(&interval_1, &interval_2);
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
