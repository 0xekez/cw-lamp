#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, QuerierWrapper,
    Response, StdResult, Storage,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakeChangeHook};
use crate::state::{CW721_STAKED, CWA, PREFERENCES};

const CONTRACT_NAME: &str = "crates.io:liesurely-lamp";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cw721_staked = deps.api.addr_validate(&msg.cw721_staked)?;
    CW721_STAKED.save(deps.storage, &cw721_staked)?;
    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("dao_voting_cw721_staked", cw721_staked))
}

pub fn member_weight(
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    who: &Addr,
    height: u64,
) -> StdResult<u64> {
    let voting = CW721_STAKED.load(storage)?;
    let response: dao_interface::voting::VotingPowerAtHeightResponse = querier.query_wasm_smart(
        &voting,
        &dao_interface::voting::Query::VotingPowerAtHeight {
            address: who.to_string(),
            height: Some(height),
        },
    )?;
    Ok(response.power.u128() as u64)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetPreference { preference } => {
            let weight =
                member_weight(deps.storage, &deps.querier, &info.sender, env.block.height)?;
            if let Some(preference) = PREFERENCES.may_load(deps.storage, &info.sender)? {
                CWA.remove(deps.storage, weight, preference)?;
            }
            PREFERENCES.save(deps.storage, &info.sender, &preference)?;
            CWA.add(deps.storage, weight, preference)?;
            Ok(Response::default()
                .add_attribute("method", "set_preference")
                .add_attribute("preference", preference.to_string()))
        }
        ExecuteMsg::StakeChangeHook(StakeChangeHook::Stake { addr, .. }) => {
            let member = deps.api.addr_validate(&addr)?;
            let weight = member_weight(deps.storage, &deps.querier, &member, env.block.height + 1)?;
            if let Some(preference) = PREFERENCES.may_load(deps.storage, &member)? {
                if weight != 0 {
                    let old = weight - 1;
                    let new = weight;
                    if old != 0 {
                        CWA.remove(deps.storage, old, preference)?;
                    }
                    CWA.add(deps.storage, new, preference)?;
                }
            }
            Ok(Response::default())
        }
        ExecuteMsg::StakeChangeHook(StakeChangeHook::Unstake { addr, token_ids }) => {
            let member = deps.api.addr_validate(&addr)?;
            let weight = member_weight(deps.storage, &deps.querier, &member, env.block.height + 1)?;
            if let Some(preference) = PREFERENCES.may_load(deps.storage, &member)? {
                let old = weight + token_ids.len() as u64;
                let new = weight;
                CWA.remove(deps.storage, old, preference)?;
                CWA.add(deps.storage, new, preference)?;
            }
            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Setpoint {} => to_binary(&CWA.average(deps.storage)?),
        QueryMsg::Preferences { start_after, limit } => {
            let start_after = start_after
                .map(|start_after| deps.api.addr_validate(&start_after))
                .transpose()?;
            let items = PREFERENCES.range(
                deps.storage,
                start_after.as_ref().map(Bound::exclusive),
                None,
                Order::Ascending,
            );
            let res: Vec<(Addr, Decimal)> = match limit {
                Some(limit) => items.take(limit as usize).collect::<StdResult<_>>()?,
                None => items.collect::<StdResult<_>>()?,
            };
            to_binary(&res)
        }
    }
}
