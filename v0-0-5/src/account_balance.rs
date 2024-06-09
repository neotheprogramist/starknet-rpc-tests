use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AccountBalanceResponseV0_0_5 {
    pub amount: (u64, u64, u64),
    pub unit: String,
}
