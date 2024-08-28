pub mod txn_validation;
use txn_validation::validate::validate_txn_json;

fn main() {
    let public_key = "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b";
    let chain_id = "0x534e5f5345504f4c4941";
    match validate_txn_json("t9n/testdata/invoke/invoke_txn_v3_rpc_test.json", &public_key, &chain_id) {
        Ok(_) => println!("JSON is valid"),
        Err(e) => println!("Validation error: {}", e),
    }
}
