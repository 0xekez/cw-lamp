use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {
    /// The cw4 contract to assign voting power based on.
    pub cw721_staked: String,
}

#[cw_serde]
pub enum StakeChangeHook {
    // Staking contract hooks.
    // ref: `dao-contracts/contracts/voting/dao-voting-cw721-staked/src/hooks.rs`
    Stake {
        addr: String,
        token_id: String,
    },
    Unstake {
        addr: String,
        token_ids: Vec<String>,
    },
}

#[cw_serde]
pub enum ExecuteMsg {
    SetPreference { preference: Decimal },
    StakeChangeHook(StakeChangeHook),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<(String, ::cosmwasm_std::Decimal)>)]
    Preferences {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(::cosmwasm_std::Decimal)]
    Setpoint {},
}
