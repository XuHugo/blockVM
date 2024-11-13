mod types;
mod traits;

use std::{collections::BTreeMap, fs};

use anyhow::{ bail, ensure, anyhow, Result};
use aptos_types::{state_store::state_key::StateKey, vm_status::VMStatus, write_set::WriteOp};
use block_executor::{executor::MVHashMapView, scheduler::Scheduler, mutex::Mutex};
use concordium_contracts_common::{AccountAddress, Amount, Address};
use geeco_mvhashmap::MVHashMap;
use storage::{StorageInstanceRef, key_value_db::KeyValueDB};
use wasmtime::{
    Trap,
    Extern,
    Linker,
    Caller,
    Engine, 
    Module, 
    Store,
    Val::{I64, I32},
};

use executor::{types::{Context, ADDRESS_SIZE, GAS_SCALE, GAS_ENV_FUNC_BASE, GAS_INIT_FUNC_BASE, ContextStm, WasmTransactionOutput, VMKind, ContractResult, ContractKind, ExecKind}, wasmtime2::execute_transaction, exec::{Runtime, Executor, preprocessing}, VM, utils::{from_json_contract, to_json_result}, Contract};
use types::{BlockGeecoVM, MockStateView, GeecoTransactionOutput};

use crate::types::PreprocessedTransaction;



fn execute_transaction_stm( ) {
    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000002");
    //account3
    let address3 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000003");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000009");

    let db = StorageInstanceRef.write().account_db();

    let c_name = String::from("evidence2");
    // let func_name = String::from("call");
    // let mut f_name = String::new();
    // f_name = format!("{}.{}", c_name, func_name);
    let f_name = format!("init_{}", c_name);

    let modules = fs::read("./wasm_file_test/evidence2.wasm").unwrap();
    let engine = wasmtime::Engine::default();
    let aot_bytes = match engine.precompile_module(&modules) {
        Ok(b) => b,
        Err(e) => {
            return
        },
    };
    //state:                 [0, 0, 0, 0, 0, 0, 0, 0, 50, 100, 52, 55, 98, 56, 52, 101, 56, 52, 53, 98, 55, 101, 51, 54, 50, 51, 51, 53, 53, 102, 48, 52, 102, 105, 98, 48, 48, 48, 48, 48, 48, 49]
    let state:Vec<u8> = vec![48, 120, 102, 54, 98, 48, 50, 97, 50, 100, 52, 55, 98, 56, 52, 101, 56, 52, 53, 98, 55, 101, 51, 54, 50, 51, 51, 53, 53, 102, 48, 52, 102, 105, 98, 48, 48, 48, 48, 48, 48, 49];
    let state = vec![];
    //generate schema and store
    preprocessing(&modules, &contract_address, &db).unwrap();
    //init
    let init_param = fs::read("./wasm_file_test/evidence_init.json").unwrap();

    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    let schema_addr = msp::HashInstanceRef.read().hash(&state_addr);
    // let mut schema_addr = contract_address.clone().into_bytes();
    // schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, &init_param, String::from("evidence2"), ExecKind::Init, String::new()).unwrap();



    let vm = BlockGeecoVM::new();
    let sv = MockStateView();
    let txn = PreprocessedTransaction{ 
        kind:ExecKind::Init,
        sender: AccountAddress::from(address1.clone()), 
        dest: AccountAddress::from(contract_address.clone()), 
        contract_name: c_name, 
        func_name: f_name, 
        balance: 100, 
        owner: AccountAddress::from(address1.clone()), 
        gas_limit: 100000, 
        gas: true, 
        code:aot_bytes, 
        param: vec![], 
        amount: 9,
        index: 1, 
        state:state.clone(),
    };
    
    let scheduler = Scheduler::new(1);
    let versioned_data_cache = MVHashMap::new();
    let state_view = MVHashMapView {
        versioned_map: &versioned_data_cache,
        txn_idx: 1,
        scheduler: &scheduler,
        captured_reads: Mutex::new(Vec::new()),
    };


    let context:ContextStm<StateKey> = ContextStm::new_call(
        ExecKind::Init,
        &txn.contract_name,
        &txn.func_name,
        &txn.param,
        concordium_contracts_common::Amount { micro_gtu: txn.balance },
        txn.sender,
        txn.owner,
        txn.dest,
        txn.gas_limit,
        txn.gas,
        &state_view,
        &state,
    );
    match execute_transaction(&txn.func_name, context, &txn.code, 0){
        Ok(o) => match o{
            WasmTransactionOutput::Success { state, logs, actions, returndata, remaining_gas , write_set } => {
                println!("");
            },
            WasmTransactionOutput::Reject { reason, remaining_gas } => {println!("");},
        },
        Err(e) => {
            println!("execute_transaction:{}",e);
        },
    }
}

fn new_tx(path:&str, db:KeyValueDB,contract_address:String, contract_name:&str, func_name:&str, address_sender:String, code_aot:Vec<u8>)->PreprocessedTransaction{
    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    let schema_addr = msp::HashInstanceRef.read().hash(&state_addr);
    //parameter json
    let balance_param = fs::read(path).unwrap();
    //let mut schema_addr = contract_address.clone().into_bytes();
    //schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, &balance_param, String::from(contract_name), ExecKind::Call, String::from(func_name)).unwrap();


    let c_name = String::from(contract_name);
    let func_name = String::from(func_name);

    let mut f_name = String::new();
    f_name = format!("{}.{}", c_name, func_name);

    //vc
    let state:Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];

    //get code from db;
    let tx = PreprocessedTransaction{ 
        kind:ExecKind::Call,
        sender: AccountAddress::from(address_sender.clone()), 
        dest: AccountAddress::from(contract_address.clone()), 
        contract_name: c_name, 
        func_name: f_name, 
        balance: 100, 
        owner: AccountAddress::from(address_sender.clone()), 
        gas_limit: 100000, 
        gas: false, 
        code:code_aot, 
        param: param, 
        amount: 9,
        index: 2,
        state};    
    tx
}

fn new_tx_2(balance_param:&[u8], db:KeyValueDB,contract_address:String, contract_name:&str, func_name:&str, address_sender:String, code_aot:Vec<u8>)->PreprocessedTransaction{
    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    let schema_addr = msp::HashInstanceRef.read().hash(&state_addr);
    //parameter json

    //let mut schema_addr = contract_address.clone().into_bytes();
    //schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, balance_param, String::from(contract_name), ExecKind::Call, String::from(func_name)).unwrap();


    let c_name = String::from(contract_name);
    let func_name = String::from(func_name);

    let mut f_name = String::new();
    f_name = format!("{}.{}", c_name, func_name);

    //vc
    let state:Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];

    //get code from db;
    let tx = PreprocessedTransaction{ 
        kind:ExecKind::Call,
        sender: AccountAddress::from(address_sender.clone()), 
        dest: AccountAddress::from(contract_address.clone()), 
        contract_name: c_name, 
        func_name: f_name, 
        balance: 100, 
        owner: AccountAddress::from(address_sender.clone()), 
        gas_limit: 100000, 
        gas: false, 
        code:code_aot, 
        param: param, 
        amount: 9,
        index: 2,
        state};    
    tx
}

fn exe_block(){
    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000002");
    //account3
    let address3 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000003");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000009");

    let db = StorageInstanceRef.write().account_db();


    let modules = fs::read("./wasm_file_test/evidence2.wasm").unwrap();


    //generate schema and store
    preprocessing(&modules, &contract_address, &db).unwrap();
    //store code
    let engine = wasmtime::Engine::default();
    let aot_bytes = match engine.precompile_module(&modules) {
        Ok(b) => b,
        Err(e) => {
            return
        },
    };
    db.lock().put_bytes(contract_address.as_bytes(), &aot_bytes);


    //init
    let init_param = fs::read("./wasm_file_test/evidence_init.json").unwrap();

    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    let schema_addr = msp::HashInstanceRef.read().hash(&state_addr);
    // let mut schema_addr = contract_address.clone().into_bytes();
    // schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, &init_param, String::from("evidence2"), ExecKind::Init, String::new()).unwrap();
    let param = vec![];

    let c_name = String::from("evidence2");
    // let func_name = String::from("init");
    // let mut f_name = String::new();
    // f_name = format!("{}.{}", c_name, func_name);

    let f_name = format!("init_{}", c_name);

   //vc
    //let state:Vec<u8> = vec![48, 120, 102, 54, 98, 48, 50, 97, 50, 100, 52, 55, 98, 56, 52, 101, 56, 52, 53, 98, 55, 101, 51, 54, 50, 51, 51, 53, 53, 102, 48, 52, 102, 105, 98, 48, 48, 48, 48, 48, 48, 49];
    let state = vec![];
    let vm = BlockGeecoVM::new();
    let sv = MockStateView();

    //get code from db;
    let tx1 = PreprocessedTransaction{ 
        kind:ExecKind::Init,
        sender: AccountAddress::from(address1.clone()), 
        dest: AccountAddress::from(contract_address.clone()), 
        contract_name: c_name, 
        func_name: f_name, 
        balance: 100, 
        owner: AccountAddress::from(address1.clone()), 
        gas_limit: 100000, 
        gas: false, 
        code:aot_bytes.clone(), 
        param: param, 
        amount: 9,
        index: 1,
        state};
    let txs = vec![tx1];

    let ret = match  BlockGeecoVM::execute_block(txs, &sv, 2){
        Ok(v) => v,
        Err(e) => {
            println!("Main error : {}", e);
            return 
        },
    };

    for item in ret{
        println!("geecooutpt:{:?}",item);
    }

    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    db.lock().put_bytes(&state_addr, &vec![0, 0, 0, 0, 0, 0, 0, 0]);

    let tx0 = new_tx("./wasm_file_test/evidence2_rwset0.json",db.clone(), contract_address.clone(),"evidence2","write",address1.clone(),aot_bytes.clone());
    let tx1 = new_tx("./wasm_file_test/evidence2_rwset1.json",db.clone(), contract_address.clone(),"evidence2","rwet",address1.clone(),aot_bytes.clone());
    let tx_get = new_tx("./wasm_file_test/evidence2_get.json",db.clone(),contract_address.clone(),"evidence2","read",address1.clone(),aot_bytes.clone());

    let mut txs :Vec<PreprocessedTransaction>= vec![];
    txs.push(tx0);
    for i in 1..101{
        //println!("{}",i);
        let pj = json!({
            "key":"202305081530",
            "value":i,
        });
        let pjs = pj.to_string();
        let p2 = pjs.as_bytes();
        let func_json: serde_json::Value = serde_json::from_slice(p2).unwrap();
        println!("{:?}",func_json);
        let tx = new_tx_2(p2,db.clone(), contract_address.clone(),"evidence2","rwet",address1.clone(),aot_bytes.clone());
        txs.push(tx);
    }
    txs.push(tx_get);

    let ret = match  BlockGeecoVM::execute_block(txs, &sv, 4){
        Ok(v) => v,
        Err(e) => {
            println!("Main error : {}", e);
            return 
        },
    };

    for item in ret{
        println!("geecooutpt:{:?}",item);
        if item.data.returndata.len()>0{
            let ret = to_json_result(&schema, &item.data.returndata,  String::from("evidence2"),  String::from("read")).unwrap();
            println!("evidence get :{:?}", ret);
        }
        
    }
    let state_addr = msp::HashInstanceRef.read().hash(&contract_address.as_bytes());
    db.lock().put_bytes(&state_addr, &vec![]);

}

use serde_json::{Value, json};
use serde::{Deserialize,Serialize};
#[derive( Deserialize, Serialize)]
struct RWset {
    key:  String,
    value:  u64,
}

fn test_json(){
    
    let p_json = r#"
    {
        "key":  "202305081530",
        "value":   0
      }
    "#;
    let p1 = p_json.as_bytes();
      let v :u64= 0;
    let pj = json!({
        "key":"202305081530",
        "value":v,
    });
    let pjs = pj.to_string();
    let p2 = pjs.as_bytes();

    let set0 = RWset{
        key:"202305081530".to_string(),
        value:0,
    };
    let p3 = serde_json::to_vec(&set0).unwrap();
    let balance_param = fs::read("./wasm_file_test/evidence2_rwset0.json").unwrap();
    println!("{:?}",p3);
    println!("{:?}",balance_param);
    let func_json: serde_json::Value = serde_json::from_slice(p1).unwrap();
    println!("{:?}",func_json);
    let func_json: serde_json::Value = serde_json::from_slice(p2).unwrap();
    println!("{:?}",func_json);
    let func_json: serde_json::Value = serde_json::from_slice(&p3).unwrap();
    println!("{:?}",func_json);
    let func_json: serde_json::Value = serde_json::from_slice(&balance_param).unwrap();
    println!("{:?}",func_json);
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");

    for i in 0..100{
        //println!("{}",i);
        let pj = json!({
            "key":"202305081530",
            "value":i,
        });
        let pjs = pj.to_string();
        let p2 = pjs.as_bytes();
        let func_json: serde_json::Value = serde_json::from_slice(p2).unwrap();
        println!("{:?}",func_json);
    }
}

fn main() {
    println!("Hello, world!");
    //execute_transaction_stm();
    exe_block();

    //main_evidence();
    //test_json();
}
