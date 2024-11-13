use std::{
    fs,
};
use concordium_contracts_common::{ Amount, Address, AccountAddress};
use executor::{types::{Context, ContractResult, ContractKind, ExecKind, VMKind}, exec::Executor, Contract, utils::{from_json_contract, to_json_result}};
use storage::{StorageInstanceRef};
use serde_json::{Value};

#[test]
fn erc721_run() {
    let vm_kind = VMKind::WasmTime; //GeeCo     WasmTime

    //wasm
    let modules = fs::read("./wasm_file_test/erc721.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8d50d9e3af");
    //account2
    let address2 = String::from("erc721hkMueLN92fL1C39nCXtxNR9dnLyD6ypAsto2");
    //contract address
    let contract_address = String::from("erc721hkMueLN92fL1C39nCXtxNR9dnLyD6ypAsto3");

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("erc721", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()), 1000_000, true);
    let mut contract = Executor { db: StorageInstanceRef.write().account_db(), context: init_ctx, contractkind: ContractKind::Concordium, vm_kind: VMKind::GeeCo };

    //generate schema and store
    contract.preprocessing(module).unwrap();

    //parameter json
    let init_param = fs::read("./wasm_file_test/erc721_init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema: Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(&schema, &init_param, String::from("erc721"), ExecKind::Init, String::new()).unwrap();

    let balance = Amount::from_gtu(0);
    let init_ctx = Context::new_init("erc721", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()), 1000_000, true);
    let mut contract = Executor { db: StorageInstanceRef.write().account_db(), context: init_ctx, contractkind: ContractKind::Concordium, vm_kind: vm_kind.clone() }; //GeeCo

    let ret = contract.exec(Some(module), 0);
    match ret {
        Ok(r) => {
            match r {
                ContractResult::Success { remaining_energy, actions, event } => println!("erc721 init success :{:?}", remaining_energy),
                ContractResult::Data { data, remaining_energy, event } => println!("erc721 init data :{:?}", remaining_energy),
                ContractResult::Reject { reason, remaining_energy } => println!("erc721 init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy => println!("erc721 init outofe."),
            };
        },
        Err(err) => println!("erc721 init err :{:?}", err),
    }


    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/erc721_mint.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("erc721"), ExecKind::Call, String::from("mint"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("param error:{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "mint",
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
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium , vm_kind:vm_kind.clone()}; //GeeCo
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("erc721 mint success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> println!("erc721 mint data :{:?}", data),
                    ContractResult::Reject{ reason, remaining_energy } => println!("erc721 mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("erc721 mint outofe."),
                };
            },
            Err(err)=>println!("erc721 mint err :{:?}", err),
        }

    }

    {
        //mintsssss~~~~~~~~~~~~
        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/erc721_mints.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("erc721"), ExecKind::Call, String::from("mints"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("param error:{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "mints",
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
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium , vm_kind:vm_kind.clone()}; //GeeCo
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("erc721 mints success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> println!("erc721 mints data :{:?}", data),
                    ContractResult::Reject{ reason, remaining_energy } => println!("erc721 mints reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("erc721 mints outofe."),
                };
            },
            Err(err)=>println!("erc721 mints err :{:?}", err),
        }

    }

    {
        //balanceof~~~~~~~~~~~~
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/erc721_balanceof.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("erc721"), ExecKind::Call, String::from("balanceOf"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("param error:{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "balanceOf",
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
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium , vm_kind:vm_kind.clone()}; //GeeCo
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("erc721 balanceOf success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {//println!("erc721 receive data :{:?}", data);
                        let ret = to_json_result(&schema, &data.returndata, String::from("erc721"), String::from("balanceOf")).unwrap();
                        println!("erc721 balanceOf :{:?}", ret);
                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("erc721 balanceOf reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("erc721 balanceOf outofe."),
                };
            },
            Err(err)=>println!("erc721 balanceOf err :{:?}", err),
        }

    }

    {
        //transfer_from~~~~~~~~~~~~
        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/erc721_transfer.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("erc721"), ExecKind::Call, String::from("transferFrom"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("param error:{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "transferFrom",
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
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium , vm_kind:vm_kind.clone()}; //GeeCo
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("erc721 transferFrom success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {//println!("erc721 receive data :{:?}", data);
                        let ret = to_json_result(&schema, &data.returndata, String::from("erc721"), String::from("transferFrom")).unwrap();
                        println!("erc721 transferFrom :{:?}", ret);
                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("erc721 transferFrom reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("erc721 transferFrom outofe."),
                };
            },
            Err(err)=>println!("erc721 transferFrom err :{:?}", err),
        }

    }

    {
        //ownerof~~~~~~~~~~~~
        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/erc721_ownerOf.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema:Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(&schema, &transfer_param, String::from("erc721"), ExecKind::Call, String::from("ownerOf"));
        let param:Vec<u8> = match p_r {
            Ok(p)=> p,
            Err(e)=> {
                println!("param error:{:?}",e);
                return;
            },
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state:Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "ownerOf",
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
        let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium , vm_kind:vm_kind.clone()}; //GeeCo
        let ret = contract.exec(None, 0);
        match ret{
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r{
                    ContractResult::Success { remaining_energy, actions, event }=> println!("erc721 ownerOf success :{:?}", remaining_energy),
                    ContractResult::Data { data,remaining_energy, event }=> {//println!("erc721 receive data :{:?}", data);
                        let ret = to_json_result(&schema, &data.returndata, String::from("erc721"), String::from("ownerOf")).unwrap();
                        println!("erc721 ownerOf :{:?}", ret);
                    },
                    ContractResult::Reject{ reason, remaining_energy } => println!("erc721 ownerOf reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy=> println!("erc721 ownerOf outofe."),
                };
            },
            Err(err)=>println!("erc721 ownerOf err :{:?}", err),
        }

    }

    {
        //info!!!!!!!!!!!
        //parameter bytes
        let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
        //parameter json
        //let transfer_param = fs::read("./wasm_file_test/erc721_info.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "erc721",
            "info",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true
        );
        let mut contract = Executor { db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind: ContractKind::Concordium, vm_kind: vm_kind.clone() }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success { remaining_energy, actions, event } => println!("erc721 info success :{:?}", remaining_energy),
                    ContractResult::Data { data, remaining_energy, event } =>
                        {//println!("erc721 receive data :{:?}", data);
                            let ret = to_json_result(&schema, &data.returndata, String::from("erc721"), String::from("info")).unwrap();
                            println!("erc721 info :{:?}", ret);
                        },
                    ContractResult::Reject { reason, remaining_energy } => println!("erc721 info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc721 info outofe."),
                };
            },
            Err(err) => println!("erc721 info err :{:?}", err),
        }
    }

}