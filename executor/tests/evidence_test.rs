use std::{
    fs,
};
use concordium_contracts_common::{ Amount, Address, AccountAddress};
use executor::{types::{Context, ContractResult, ContractKind, ExecKind, VMKind}, exec::Executor, Contract, utils::{from_json_contract, to_json_result}};
use storage::{StorageInstanceRef};
use serde_json::{Value};

#[test]
fn evidence_run(){
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let modules = fs::read("./wasm_file_test/evidence.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04evidence01");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04evidence02");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04evidence03");

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("evidence", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()),1000_000, true);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };

    //generate schema and store
    contract.preprocessing(module).unwrap();

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];

    //parameter json
    let init_param = fs::read("./wasm_file_test/evidence_init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, &init_param, String::from("evidence"), ExecKind::Init, String::new()).unwrap();

    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("evidence", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()),1000_000, true);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium , vm_kind: vm_kind.clone()};

    let ret = contract.exec(Some(module), 0);
    match ret{
        Ok(r) => {
            //println!("test:erc20 ok :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("evidence init success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> println!("evidence init data :{:?}", remaining_energy),
                ContractResult::Reject{ reason, remaining_energy } => println!("evidence init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("evidence init outofe."),
            };
        },
        Err(err)=>println!("evidence init err :{:?}", err),
    }

    {
        let balance =  Amount::from_gtu(0);

        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/evidence_create.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("evidence"), ExecKind::Call, String::from("create"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "evidence",
            "create",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true
        );
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("evidence c success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> println!("evidence c data :{:?}", data),
                    ContractResult::Reject{ reason, remaining_energy } => println!("evidence c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("evidence c outofe."),
                };
            },
            Err(err)=>println!("evidence c err :{:?}", err),
        }

    }

    {
        let balance =  Amount::from_gtu(10000);
        //parameter bytes
        let param = [52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 49];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_set.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(&schema, &balance_param, String::from("evidence"), ExecKind::Call, String::from("set")).unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "evidence",
            "set",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true
        );
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("evidence set success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(&schema, &data.returndata,  String::from("evidence"),  String::from("set")).unwrap();
                        println!("evidence set :{:?}", ret);

                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("evidence set reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("evidence set outofe."),
                };
            },
            Err(err)=>println!("evidence set err :{:?}", err),
        }
    }

    {
        let balance =  Amount::from_gtu(0);
        //parameter bytes
        let param = [52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 49];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_get.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(&schema, &balance_param, String::from("evidence"), ExecKind::Call, String::from("get")).unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "evidence",
            "get",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true
        );
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("evidence get success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(&schema, &data.returndata,  String::from("evidence"),  String::from("get")).unwrap();
                        println!("evidence get :{:?}", ret);

                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("evidence get reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("evidence get outofe."),
                };
            },
            Err(err)=>println!("evidence get err :{:?}", err),
        }
    }

    {
        let balance =  Amount::from_gtu(0);
        //parameter bytes
        let param = [52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 49];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_info.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(&schema, &balance_param, String::from("evidence"), ExecKind::Call, String::from("info")).unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "evidence",
            "info",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true
        );
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("evidence info success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(&schema, &data.returndata,  String::from("evidence"),  String::from("info")).unwrap();
                        println!("evidence info :{:?}", ret);

                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("evidence info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("evidence info outofe."),
                };
            },
            Err(err)=>println!("evidence info err :{:?}", err),
        }
    }

}