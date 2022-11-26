use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub create_slave: bool,
    pub slave_contract_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    RedirectFunds {},
    TakeMyNativeMoney {},
    Withdraw { admin: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetMasterBalance)]
    GetMasterBalance {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetMasterBalance {
    pub count: u128,
}
