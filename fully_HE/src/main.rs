/* ===================== IMPORTS ===================== */
use tfhe::prelude::*;
use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};

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
    let interval= Interval::new(6, 7);

    let config = ConfigBuilder::default().build();

 
    let (client_key, server_key) = generate_keys(config);
    
    let clear_a = 7u8;
    let clear_b= 128u8;

    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);

    set_server_key(server_key);

    let result = a + b;

    let decrypted_result: u8 = result.decrypt(&client_key);

    let clear_result = clear_a + clear_b;

    println!("The HE result is: {:?}", decrypted_result);
    println!("The clear result is: {:?}", clear_result);
}

