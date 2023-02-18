use cosmwasm_std::{Addr, Decimal, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

const ADDR1: &str = "addr1";
const ADDR2: &str = "addr2";
const ADDR3: &str = "addr3";
const ADDR4: &str = "addr4";

fn cw4_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
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

#[test]
fn test_incrementing() {
    let mut app = App::default();
    let cw4_code = app.store_code(cw4_contract());
    let lamp_code = app.store_code(lamp_contract());

    let members = vec![
        cw4::Member {
            addr: ADDR1.to_string(),
            weight: 4,
        },
        cw4::Member {
            addr: ADDR2.to_string(),
            weight: 3,
        },
        cw4::Member {
            addr: ADDR3.to_string(),
            weight: 2,
        },
        cw4::Member {
            addr: ADDR4.to_string(),
            weight: 1,
        },
    ];

    let cw4 = app
        .instantiate_contract(
            cw4_code,
            Addr::unchecked("meow"),
            &cw4_group::msg::InstantiateMsg {
                admin: Some("meow".to_string()),
                members,
            },
            &[],
            "cw4_group",
            None,
        )
        .unwrap();

    let lamp = app
        .instantiate_contract(
            lamp_code,
            Addr::unchecked("meow"),
            &InstantiateMsg {
                cw4: cw4.to_string(),
            },
            &[],
            "lamp",
            None,
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("meow"),
        cw4,
        &cw4_group::msg::ExecuteMsg::AddHook {
            addr: lamp.to_string(),
        },
        &[],
    )
    .unwrap();

    // preferences: [1, 10, 8, 3]
    // weights    : [4, 3, 2, 1]
    app.execute_contract(
        Addr::unchecked(ADDR1),
        lamp.clone(),
        &ExecuteMsg::SetPreference {
            preference: Decimal::percent(1),
        },
        &[],
    )
    .unwrap();

    let average: Decimal = app
        .wrap()
        .query_wasm_smart(&lamp, &QueryMsg::Setpoint {})
        .unwrap();
    assert_eq!(average, Decimal::percent(1));

    app.execute_contract(
        Addr::unchecked(ADDR2),
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
    assert!(average.to_string().starts_with("0.04"));
}
