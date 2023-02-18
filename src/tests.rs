use anyhow::Result as AnyResult;
use cosmwasm_std::Binary;
use cosmwasm_std::{Addr, Decimal, Empty};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/// Address used as the owner, instantiator, and minter.
pub(crate) const CREATOR_ADDR: &str = "creator";

fn voting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        dao_voting_cw721_staked::contract::execute,
        dao_voting_cw721_staked::contract::instantiate,
        dao_voting_cw721_staked::contract::query,
    );
    Box::new(contract)
}

fn lamp_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw721_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

pub fn instantiate_cw721_base(app: &mut App, sender: &str, minter: &str) -> Addr {
    let cw721_id = app.store_code(cw721_contract());

    app.instantiate_contract(
        cw721_id,
        Addr::unchecked(sender),
        &cw721_base::InstantiateMsg {
            name: "bad kids".to_string(),
            symbol: "bad kids".to_string(),
            minter: minter.to_string(),
        },
        &[],
        "cw721_base".to_string(),
        None,
    )
    .unwrap()
}

pub(crate) struct CommonTest {
    app: App,
    module: Addr,
    nft: Addr,
}

pub(crate) fn setup_test() -> CommonTest {
    let mut app = App::default();
    let module_id = app.store_code(voting_contract());

    let nft = instantiate_cw721_base(&mut app, CREATOR_ADDR, CREATOR_ADDR);
    let module = app
        .instantiate_contract(
            module_id,
            Addr::unchecked(CREATOR_ADDR),
            &dao_voting_cw721_staked::msg::InstantiateMsg {
                owner: Some(dao_interface::Admin::Address {
                    addr: CREATOR_ADDR.to_string(),
                }),
                nft_address: nft.to_string(),
                unstaking_duration: None,
            },
            &[],
            "cw721_voting",
            None,
        )
        .unwrap();
    CommonTest { app, module, nft }
}

// Shorthand for an unchecked address.
macro_rules! addr {
    ($x:expr ) => {
        Addr::unchecked($x)
    };
}

pub fn send_nft(
    app: &mut App,
    cw721: &Addr,
    sender: &str,
    receiver: &Addr,
    token_id: &str,
    msg: Binary,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        cw721.clone(),
        &cw721_base::ExecuteMsg::<Empty, Empty>::SendNft {
            contract: receiver.to_string(),
            token_id: token_id.to_string(),
            msg,
        },
        &[],
    )
}

pub fn mint_nft(
    app: &mut App,
    cw721: &Addr,
    sender: &str,
    receiver: &str,
    token_id: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        cw721.clone(),
        &cw721_base::ExecuteMsg::Mint::<Empty, Empty>(cw721_base::MintMsg {
            token_id: token_id.to_string(),
            owner: receiver.to_string(),
            token_uri: None,
            extension: Empty::default(),
        }),
        &[],
    )
}

pub fn stake_nft(
    app: &mut App,
    cw721: &Addr,
    module: &Addr,
    sender: &str,
    token_id: &str,
) -> AnyResult<AppResponse> {
    send_nft(app, cw721, sender, module, token_id, Binary::default())
}

pub fn mint_and_stake_nft(
    app: &mut App,
    cw721: &Addr,
    module: &Addr,
    receiver: &str,
    token_id: &str,
) -> AnyResult<()> {
    mint_nft(app, cw721, CREATOR_ADDR, receiver, token_id)?;
    stake_nft(app, cw721, module, receiver, token_id)?;
    Ok(())
}

pub fn unstake_nfts(
    app: &mut App,
    module: &Addr,
    sender: &str,
    token_ids: &[&str],
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &dao_voting_cw721_staked::msg::ExecuteMsg::Unstake {
            token_ids: token_ids.iter().map(|s| s.to_string()).collect(),
        },
        &[],
    )
}

pub fn add_hook(app: &mut App, module: &Addr, sender: &str, hook: &str) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &dao_voting_cw721_staked::msg::ExecuteMsg::AddHook {
            addr: hook.to_string(),
        },
        &[],
    )
}

#[test]
fn lamp_lamp_lamp() {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    let lamp_code = app.store_code(lamp_contract());

    let lamp = app
        .instantiate_contract(
            lamp_code,
            Addr::unchecked("meow"),
            &InstantiateMsg {
                cw721_staked: module.to_string(),
            },
            &[],
            "lamp",
            None,
        )
        .unwrap();

    add_hook(&mut app, &module, CREATOR_ADDR, lamp.as_str()).unwrap();

    app.execute_contract(
        Addr::unchecked("meow"),
        lamp.clone(),
        &ExecuteMsg::SetPreference {
            preference: Decimal::percent(1),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("ekez"),
        lamp.clone(),
        &ExecuteMsg::SetPreference {
            preference: Decimal::percent(10),
        },
        &[],
    )
    .unwrap();

    let average: Decimal = app
        .wrap()
        .query_wasm_smart(&lamp, &QueryMsg::Setpoint {})
        .unwrap();
    assert_eq!(average, Decimal::zero());

    // ekez gains three voting power
    mint_and_stake_nft(&mut app, &nft, &module, "ekez", "1").unwrap();
    mint_and_stake_nft(&mut app, &nft, &module, "ekez", "2").unwrap();
    mint_and_stake_nft(&mut app, &nft, &module, "ekez", "3").unwrap();

    // meow gains 4
    mint_and_stake_nft(&mut app, &nft, &module, "meow", "4").unwrap();
    mint_and_stake_nft(&mut app, &nft, &module, "meow", "5").unwrap();
    mint_and_stake_nft(&mut app, &nft, &module, "meow", "6").unwrap();
    mint_and_stake_nft(&mut app, &nft, &module, "meow", "7").unwrap();

    let average: Decimal = app
        .wrap()
        .query_wasm_smart(&lamp, &QueryMsg::Setpoint {})
        .unwrap();
    assert!(average.to_string().starts_with("0.04857"));

    // ekez looses a voting power.
    unstake_nfts(&mut app, &module, "ekez", &["2"]).unwrap();

    let average: Decimal = app
        .wrap()
        .query_wasm_smart(&lamp, &QueryMsg::Setpoint {})
        .unwrap();
    assert!(average.to_string().starts_with("0.04"));
}
