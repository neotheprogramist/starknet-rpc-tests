pub mod txn_validation;
use txn_validation::validate::validate_txn_json;

fn main() {
    match validate_txn_json("t9n/testdata/invoke/invoke_txn_v1.json") {
        Ok(_) => println!("JSON is valid"),
        Err(e) => println!("Validation error: {}", e),
    }
}
