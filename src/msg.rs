use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {
    /// The cw4 contract to assign voting power based on.
    pub cw4: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetPreference { preference: Decimal },
    MemberChangedHook { diffs: Vec<cw4::MemberDiff> },
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
