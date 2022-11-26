use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub slave_contract_addr: Option<Addr>,
    pub admin: Addr,
}

pub const STATE: Item<State> = Item::new("state");
