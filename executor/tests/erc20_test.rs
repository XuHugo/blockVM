use std::{
    fs,
    time::Instant,
};
use concordium_contracts_common::{ Amount, Address, AccountAddress};
use executor::{types::{Context, ContractResult, ContractKind, ExecKind, VMKind}, exec::Executor, Contract, utils::{from_json_contract, to_json_result, to_json_event}};
use storage::{StorageInstanceRef};
use serde_json::{Value};

#[test]
fn erc20_run(){
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let c = AccountAddress([52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116]);
    let d = String::from(c);
    println!("addr to string:{:?}",d);

    let gas = true;

    let modules = fs::read("./wasm_file_test/erc20.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ1");
    //account2
    let address2 = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ2");
    //contract address
    let contract_address = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ0");

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("erc20", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()),1000,gas);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };

    //generate schema and store
    //let ret = contract.preprocessing(module).unwrap();
    match contract.preprocessing(module) {
        Ok(r) => r,
        Err(e) =>println!("erc20 preprocessing err :{:?}", e),
    }
    if gas{
        let module = match contract.preprocessing_gas(module){
            Ok(r) => r,
            Err(e) =>return println!("erc20 preprocessing gas  err :{:?}", e),
        };
        let module = module.as_slice();
    }


    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    println!("init~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    //parameter json
    let init_param = fs::read("./wasm_file_test/init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let param = match from_json_contract(&schema, &init_param, String::from("erc20"), ExecKind::Init, String::new()){
        Ok(r) => r,
        Err(e) =>return println!("erc20 from_json_contract err :{:?}", e),
    };

    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("erc20", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()), 1000,gas);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };

    let ret = contract.exec(Some(module), 0);
    match ret{
        Ok(r) => {
            //println!("test:erc20 ok :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> {
                    let ret = to_json_event(&schema, event,  String::from("erc20"));
                    println!("event:{:?}", ret);
                    println!("erc20 init success :{:?}", remaining_energy)
                },
                ContractResult::Data { data,remaining_energy, event }=> println!("erc20 init data :{:?}", remaining_energy),
                ContractResult::Reject{ reason, remaining_energy } => println!("erc20 init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("erc20 init outofe."),
            };
        },
        Err(err)=>println!("erc20 init err :{:?}", err),
    }
    println!("time cost init:{:?} ms",start.elapsed().as_millis());

    let balance =  Amount::from_gtu(10000);
    println!("receive~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    //parameter bytes
    let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
    //parameter json
    let transfer_param = fs::read("./wasm_file_test/transfer.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let p_r = from_json_contract(&schema, &transfer_param, String::from("erc20"), ExecKind::Call, String::from("receive"));
    let param:Vec<u8> = match p_r {
        Ok(p)=> p,
        Err(e)=> {
            println!("R:from_json_contract:{:?}",e);
            return;
        },
    };

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "erc20",
        "receive",
        &param,
        &state,
        balance,
        sender,
        invoker,
        owner,
        AccountAddress::from(contract_address.clone()),
        1000,
        gas
    );
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    let ret = contract.exec(None, 0);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> {

                    // for log in event.logs{
                    //     to_json_event(&schema, &log,  String::from("erc20"));
                    // }
                    let ret = to_json_event(&schema, event,  String::from("erc20"));
                    println!("event:{:?}", ret);
                    println!("erc20 receive success :{:?}", remaining_energy)
                },
                ContractResult::Data { data,remaining_energy, event }=> println!("erc20 receive data :{:?}", data),
                ContractResult::Reject{ reason, remaining_energy } => println!("erc20 receive reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("erc20 receive outofe."),
            };
        },
        Err(err)=>println!("erc20 receive err :{:?}", err),
    }
    println!("time cost receive:{:?} ms",start.elapsed().as_millis());




    println!("setdata~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    let balance =  Amount::from_gtu(10000);
    //parameter bytes
    let param = [52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 49];

    //parameter json
    let balance_param = fs::read("./wasm_file_test/erc20_setdata.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let param = from_json_contract(&schema, &balance_param, String::from("erc20"), ExecKind::Call, String::from("setdata")).unwrap();

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "erc20",
        "setdata",
        &param,
        &state,
        balance,
        sender,
        invoker,
        owner,
        AccountAddress::from(contract_address.clone()),
        1000,
        gas
    );
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    let ret = contract.exec(None, 0);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("erc20 setdata success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> {
                    //old data;
                    println!("erc20 setdata data :{:?}", data);
                    //json data;
                    // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                    // println!("erc20 balance data :{:#?}",rr.to_string());

                    //concordium json data
                    let ret = to_json_result(&schema, &data.returndata,  String::from("erc20"),  String::from("setdata")).unwrap();
                    println!("erc20 setdata :{:?}", ret);

                },
                ContractResult::Reject{ reason, remaining_energy } => println!("erc20 setdata reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("erc20 setdata outofe."),
            };
        },
        Err(err)=>println!("erc20 setdata err :{:?}", err),
    }
    println!("time cost setddata:{:?} ms",start.elapsed().as_millis());




    let balance =  Amount::from_gtu(10000);
    //parameter bytes
    let param = [52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 49];
    println!("getdata~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    //parameter json
    let balance_param = fs::read("./wasm_file_test/erc20_getdata.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let param = from_json_contract(&schema, &balance_param, String::from("erc20"), ExecKind::Call, String::from("getdata")).unwrap();

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "erc20",
        "getdata",
        &param,
        &state,
        balance,
        sender,
        invoker,
        owner,
        AccountAddress::from(contract_address.clone()),
        1000,
        gas
    );
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    let ret = contract.exec(None, 0);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("erc20 getdata success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> {
                    //old data;
                    println!("erc20 getdata data :{:?}", data);
                    //json data;
                    // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                    // println!("erc20 balance data :{:#?}",rr.to_string());

                    //concordium json data
                    let ret = to_json_result(&schema, &data.returndata,  String::from("erc20"),  String::from("getdata")).unwrap();
                    println!("erc20 getdata :{:#?}", ret);
                    println!("erc20 gas :{:?}", remaining_energy);

                },
                ContractResult::Reject{ reason, remaining_energy } => println!("erc20 getdata reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("erc20 getdata out of energy."),
            };
        },
        Err(err)=>println!("erc20 getdata err :{:?}", err),
    }
    println!("time cost getdata:{:?} ms",start.elapsed().as_millis());
    println!("info~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    //parameter json
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let param = from_json_contract(&schema, &[], String::from("erc20"), ExecKind::Call, String::from("info")).unwrap();

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "erc20",
        "info",
        &param,
        &state,
        balance,
        sender,
        invoker,
        owner,
        AccountAddress::from(contract_address.clone()),
        1000,
        gas
    );
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    let ret = contract.exec(None, 0);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("erc20 info success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> {
                    //old data;
                    //println!("erc20 balance data :{:?}", data);
                    //json data;
                    // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                    // println!("erc20 balance data :{:#?}",rr.to_string());

                    //concordium json data
                    let ret = to_json_result(&schema, &data.returndata,  String::from("erc20"),  String::from("info")).unwrap();
                    println!("erc20 info :{:?}____gas:{:?}", ret,remaining_energy);

                },
                ContractResult::Reject{ reason, remaining_energy } => println!("erc20 info reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("erc20 info outofe."),
            };
        },
        Err(err)=>println!("erc20 info err :{:?}", err),
    }
    println!("time cost info:{:?} ms",start.elapsed().as_millis());
}