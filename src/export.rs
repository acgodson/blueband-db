use blueband_rust::*;

use candid::export_service;

export_service!(); // ✅ This generates the function export_service()

fn main() {
    println!("{}", __export_service()); // Call the correctly generated function
}
