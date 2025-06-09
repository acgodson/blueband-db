use blueband_rust::*;

use candid::export_service;

export_service!(); 

fn main() {
    println!("{}", __export_service()); 
}