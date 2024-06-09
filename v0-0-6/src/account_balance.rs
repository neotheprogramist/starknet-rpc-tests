use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AccountBalanceResponseV0_0_6 {
    pub amount: String,
    pub unit: String,
}
