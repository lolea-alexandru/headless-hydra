use rand::{Rng, SeedableRng, rngs::StdRng};

fn ore_setup() -> u128 {
    let mut rng = StdRng::from_os_rng();

    return rng.random();
}

fn main() {
    println!("Welcome to my first implementation of interval intersection built on top of PSI");

    // This first version of the algorithm will follow these steps: 
    // 1. Generate ORE secret key
    // 2. Encrypt intervals of Alice and Bob using the secret key
    // 3. Compute the interval intersection in O(a*b) complexity, where a = |Alice| and b = |Bob|
    // 4. Decrypt the results

    // Assumptions:
    // 1. The intervals of each party are not overlapping

    /* =========================== STEP 1 =========================== */
    let ore_key = ore_setup();

    print!("The ORE secret key is: {:?}", ore_key);
}
