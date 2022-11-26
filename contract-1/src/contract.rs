#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, WasmMsg,
};

use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const _CONTRACT_NAME: &str = "crates.io:contract-1";
const _CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SLAVE_CONTRACT_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;

    // let slave_addr = msg.slave_contract_addr.map(|a| deps.api.addr_validate(&a)).transpose()?;
    if msg.create_slave {
        let state = State {
            slave_contract_addr: None,
            admin: admin.clone(),
        };

        STATE.save(deps.storage, &state)?;

        let inst_msg = WasmMsg::Instantiate {
            code_id: msg.slave_contract_id,
            funds: vec![],
            admin: None,
            label: "contract_1".to_string(),
            msg: to_binary(&InstantiateMsg {
                admin: admin.into_string(),
                create_slave: false,
                slave_contract_id: 0,
            })?,
        };

        let reply_msg = SubMsg::reply_on_success(inst_msg, INSTANTIATE_SLAVE_CONTRACT_ID);

        return Ok(Response::new().add_submessage(reply_msg));
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RedirectFunds {} => execute::redirect_funds(deps, env, info),
        ExecuteMsg::TakeMyNativeMoney {} => execute::take_my_money(info),
        ExecuteMsg::Withdraw { admin } => execute::withdraw(deps, env, admin),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SLAVE_CONTRACT_ID {
        return Err(ContractError::Unauthorized {});
    };
    let res = parse_reply_instantiate_data(msg);
    match res {
        Ok(res) => {
            let slave_addr = deps.api.addr_validate(&res.contract_address)?;

            STATE.update(deps.storage, |mut sta| -> Result<_, ContractError> {
                sta.slave_contract_addr = Some(slave_addr);
                Ok(sta)
            })?;

            Ok(Response::new())
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

pub mod execute {
    use cosmwasm_std::{Coin, CosmosMsg};

    use super::*;

    pub fn take_my_money(info: MessageInfo) -> Result<Response, ContractError> {
        let MessageInfo {
            sender: _sender,
            funds,
        } = info;

        if funds.len() != 1 {
            return Err(ContractError::Unauthorized {});
        }
        Ok(Response::new())
    }

    pub fn redirect_funds(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if state.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if state.slave_contract_addr.is_none() {
            return Err(ContractError::Unauthorized {});
        }

        let current_balance = deps
            .querier
            .query_balance(&env.contract.address, "uJuno")?
            .amount;

        let redirect_funds_message = cosmwasm_std::BankMsg::Send {
            to_address: state.slave_contract_addr.clone().unwrap().into_string(),
            amount: vec![Coin {
                denom: "uJuno".to_string(),
                amount: current_balance,
            }],
        };

        let redirect_funds_cosmos_msg: CosmosMsg = redirect_funds_message.into();

        let execute_msg = WasmMsg::Execute {
            contract_addr: state.slave_contract_addr.unwrap().into_string(),
            msg: to_binary(&ExecuteMsg::Withdraw {
                admin: state.admin.into_string(),
            })?,
            funds: vec![],
        };

        Ok(Response::new()
            .add_message(redirect_funds_cosmos_msg)
            .add_message(execute_msg))
    }

    pub fn withdraw(deps: DepsMut, env: Env, admin: String) -> Result<Response, ContractError> {
        let admin = deps.api.addr_validate(&admin)?;
        let state = STATE.load(deps.storage)?;

        if admin != state.admin {
            return Err(ContractError::Unauthorized {});
        }

        let current_balance = deps
            .querier
            .query_balance(&env.contract.address, "uJuno")?
            .amount;

        let withdraw_funds_message = cosmwasm_std::BankMsg::Send {
            to_address: state.admin.into_string(),
            amount: vec![Coin {
                denom: "uJuno".to_string(),
                amount: current_balance,
            }],
        };

        Ok(Response::new().add_message(withdraw_funds_message))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}
