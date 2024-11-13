use std::{
    fs,
    time::Instant,
};
use concordium_contracts_common::{ Amount, Address, AccountAddress};
use executor::{types::{Context, ContractResult, ContractKind, ExecKind, VMKind}, exec::Executor, Contract, utils::{from_json_contract, to_json_result}};
use storage::{StorageInstanceRef};
use serde_json::{Value};
use wasm_chain_integration::*;
use concordium_contracts_common::*;

#[test]
fn fib_run(){
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let c = AccountAddress([52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116]);
    let d = String::from(c);
    println!("addr to string:{:?}",d);

    let modules = fs::read("./wasm_file_test/fib.wasm").unwrap();
    let module = modules.as_slice();

    //buyer
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1");
    //seller
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f041bcb36daf2");
    //arbiter
    let address3 = String::from("0xf6b02a2d47b84e845b7e3623355f041bcb36daf3");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f041bcb36daf4");

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    let balance = Amount::from_micro_gtu(0);
    let init_ctx = Context::new_init("fib", AccountAddress::from(address1.clone()), &[], balance, AccountAddress::from(contract_address.clone()), 0, false);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };

    //generate schema and store
    contract.preprocessing(module).unwrap();

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];

    //parameter json
    let init_param = fs::read("./wasm_file_test/escrow_init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);

    // let param = match from_json_contract(&schema, &init_param, String::from("fib"), ExecKind::Init, String::new()){
    //   Ok(p) => p,
    //     Err(e) => {println!("{:?}",e); [].to_vec()},
    // };

    let balance = Amount::from_micro_gtu(10000);
    let init_ctx = Context::new_init("fib", AccountAddress::from(address1.clone()), &[], balance, AccountAddress::from(contract_address.clone()),1000_000, true);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    let start = Instant::now();
    let ret = contract.exec(Some(module), 0);
    match ret{
        Ok(r) => {
            //println!("test:escrow ok :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("fib init success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> println!("fib init data :{:?}", remaining_energy),
                ContractResult::Reject{ reason, remaining_energy } => println!("fib init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("fib init outofe."),
            };
        },
        Err(err)=>println!("fib init err :{:?}", err),
    }
    println!("fib => time cost init:{:?} ms",start.elapsed().as_millis());

    println!("executor  research!");

    let init_ctx: InitContext<&[u8]> = InitContext {
        metadata:        ChainMetadata {
            slot_time: Timestamp::from_timestamp_millis(0),
            height: 2000,
            tx_hash: "txhash0xf6b02a2d47b84e845b7e3623355f041bcb36daf1".to_string(),
        },
        init_origin:     AccountAddress::from("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1".to_string()),
        sender_policies: &[],
    };

    let modules = fs::read("./wasm_file_test/fib.wasm").unwrap();
    let module = modules.as_slice();
    let name = format!("init_fib");
    let a = 0;
    let e =99999;
    let start = Instant::now();
    match invoke_init_from_source(
        &module,
        0,
        init_ctx,
        "init_fib",
        &[],
        99999,
    ){
        Ok(ret) => ret,
        Err(e) => {
            println!("geecowasm=>{:?}", e);
            return;
        },
    };
    println!("time cost init:{:?} ms",start.elapsed().as_millis());

}