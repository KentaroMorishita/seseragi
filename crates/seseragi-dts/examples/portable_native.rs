//! Native half of the WASM parity probe. All conversion stays in seseragi-dts.

#[path = "portable_wasm.rs"]
mod adapter;

use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let requests: Vec<serde_json::Value> = serde_json::from_str(&input).unwrap();
    let responses = requests
        .iter()
        .map(|request| {
            serde_json::from_str::<serde_json::Value>(&adapter::convert_declaration(
                &request.to_string(),
            ))
            .unwrap()
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&responses).unwrap());
}
