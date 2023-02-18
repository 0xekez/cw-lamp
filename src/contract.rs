#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, QuerierWrapper,
    Response, StdResult, Storage,
};
use cw2::set_contract_version;
use cw4::MemberDiff;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CW4, CWA, PREFERENCES};

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
    let cw4 = deps.api.addr_validate(&msg.cw4)?;
    CW4.save(deps.storage, &cw4)?;
    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("cw4", cw4))
}

pub fn member_weight(
    storage: &dyn Storage,
    querier: &QuerierWrapper,
    who: &Addr,
) -> StdResult<u64> {
    let cw4 = CW4.load(storage)?;
    let response: cw4::MemberResponse = querier.query_wasm_smart(
        &cw4,
        &cw4::Cw4QueryMsg::Member {
            addr: who.to_string(),
            at_height: None,
        },
    )?;
    Ok(response.weight.unwrap_or_default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetPreference { preference } => {
            let weight = member_weight(deps.storage, &deps.querier, &info.sender)?;
            if let Some(preference) = PREFERENCES.may_load(deps.storage, &info.sender)? {
                CWA.remove(deps.storage, weight, preference)?;
            }
            CWA.add(deps.storage, weight, preference)?;
            Ok(Response::default()
                .add_attribute("method", "set_preference")
                .add_attribute("preference", preference.to_string()))
        }
        ExecuteMsg::MemberChangedHook { diffs } => {
            let cw4 = CW4.load(deps.storage)?;
            if info.sender != cw4 {
                return Err(ContractError::NotCw4(info.sender));
            }
            for MemberDiff {
                key: member,
                old,
                new,
            } in diffs
            {
                let member = deps.api.addr_validate(&member)?;
                if let Some(preference) = PREFERENCES.may_load(deps.storage, &member)? {
                    let old = old.unwrap_or_default();
                    let new = new.unwrap_or_default();

                    CWA.remove(deps.storage, old, preference)?;
                    CWA.add(deps.storage, new, preference)?;
                }
            }
            Ok(Response::default().add_attribute("method", "member_changed_hook"))
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
