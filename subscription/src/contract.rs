#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SubResponse};
use crate::state::{Config, CONFIG, SUBSCRIPTIONS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        provider: msg.provider,
        price: msg.price.parse().unwrap_or(0),
        denom: msg.denom,
        interval_seconds: msg.interval_seconds,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Subscribe {} => execute_subscribe(deps, env, info),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Cancel {} => execute_cancel(deps, info),
    }
}

pub fn execute_subscribe(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let coin = info
        .funds
        .iter()
        .find(|c| c.denom == config.denom)
        .ok_or(ContractError::InsufficientFunds {})?;

    if coin.amount.u128() < config.price {
        return Err(ContractError::InsufficientFunds {});
    }

    let next_claim = env.block.time.seconds() + config.interval_seconds;
    SUBSCRIPTIONS.save(deps.storage, info.sender.as_str(), &next_claim)?;

    let msg = BankMsg::Send {
        to_address: config.provider,
        amount: vec![Coin { denom: config.denom, amount: Uint128::from(config.price) }],
    };

    Ok(Response::new().add_attribute("action", "subscribe").add_message(msg))
}

pub fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.provider {
        return Err(ContractError::Unauthorized {});
    }
    
    // In a real app we'd iterate over subs or pass the specific user. 
    // This is a simplified version where the provider claims if any are valid.
    return Err(ContractError::Unauthorized {}); // Dummy implementation
}

pub fn execute_cancel(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    SUBSCRIPTIONS.remove(deps.storage, info.sender.as_str());
    Ok(Response::new().add_attribute("action", "cancel"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Subscription { address } => to_binary(&query_sub(deps, address)?),
    }
}

fn query_sub(deps: Deps, address: String) -> StdResult<SubResponse> {
    let next_claim = SUBSCRIPTIONS.load(deps.storage, &address).unwrap_or(0);
    Ok(SubResponse { next_claim })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            provider: "provider".to_string(),
            price: "100".to_string(),
            denom: "earth".to_string(),
            interval_seconds: 3600,
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
