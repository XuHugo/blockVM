use concordium_contracts_common::{AccountAddress, Address, Amount, ContractAddress};
use executor::jvm::*;
use executor::{
    exec::Executor,
    types::{Context, Context_JVM, ContractKind, ContractResult, ExecKind, VMKind},
    utils::{from_json_contract, to_json_contract, to_json_event, to_json_result},
    Contract,
};
use serde_json::Value;
use std::{fs, sync::Arc, time::Instant};
use storage::StorageInstanceRef;

fn main() {
    let modules = fs::read("./wasm_file_test/vc.wasm").unwrap();
    let wasm_bytes = modules.as_slice();
    // println!("{:?}",module);

    let engine = wasmtime::Engine::default();
    let aot_bytes = match engine.precompile_module(wasm_bytes) {
        Ok(b) => b,
        Err(e) => return,
    };
    println!("{:?}", aot_bytes);

    //main_bench();
    //main_tbi();
    //main_jvm();
    //main_evidence();
    //mainnft();
    //main_geeco_erc20();
    //main_fib();
    wasmtime_bench("vc", "call");
}

fn main_evm() {}

fn main_tbi() {
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let modules = fs::read("./wasm_file_test/tbi.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000002");
    //account3
    let address3 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000003");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000009");

    //parameter bytes
    let param = [
        3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
    ];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init(
        "tbi",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000_000,
        true,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: vm_kind.clone(),
    };

    //generate schema and store
    contract.preprocessing(module).unwrap();
    {
        //parameter bytes
        let param = [
            3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
        ];

        //parameter json
        let init_param = fs::read("./wasm_file_test/tbi_init.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &init_param,
            String::from("tbi"),
            ExecKind::Init,
            String::new(),
        )
        .unwrap();

        let balance = Amount::from_gtu(10000);
        let init_ctx = Context::new_init(
            "tbi",
            AccountAddress::from(address1.clone()),
            &param,
            balance,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        let ret = contract.exec(Some(module), 0);
        match ret {
            Ok(r) => {
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi init success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("tbi init data :{:?}", remaining_energy),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi init reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi init outofe."),
                };
            }
            Err(err) => println!("tbi init err :{:?}", err),
        }
    }
    println!("create~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);

        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/tbi_create.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("create"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "create",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("tbi c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi c outofe."),
                };
            }
            Err(err) => println!("tbi c err :{:?}", err),
        }
    }
    {
        let balance = Amount::from_gtu(0);

        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/tbi_create.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("create"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address2.clone()));
        let invoker = AccountAddress::from(address2.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "create",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("tbi c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi c outofe."),
                };
            }
            Err(err) => println!("tbi c err :{:?}", err),
        }
    }
    {
        let balance = Amount::from_gtu(0);

        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/tbi_create.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("create"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address3.clone()));
        let invoker = AccountAddress::from(address3.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "create",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("tbi c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi c outofe."),
                };
            }
            Err(err) => println!("tbi c err :{:?}", err),
        }
    }
    println!("frozen 0~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(10000);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/tbi_frozen.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("frozen"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "frozen",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi frozen success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("tbi"),
                            String::from("frozen"),
                        )
                        .unwrap();
                        println!("tbi frozen :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi frozen reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi frozen outofe."),
                };
            }
            Err(err) => println!("tbi frozen err :{:?}", err),
        }
    }
    println!("info~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/tbi_info.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("info"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "info",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi info success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("tbi"),
                            String::from("info"),
                        )
                        .unwrap();
                        println!("tbi info :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi info outofe."),
                };
            }
            Err(err) => println!("tbi info err :{:?}", err),
        }
    }
    println!("frozen1~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(10000);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/tbi_frozen1.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("frozen"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "frozen",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi frozen success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("tbi"),
                            String::from("frozen"),
                        )
                        .unwrap();
                        println!("tbi frozen :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi frozen reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi frozen outofe."),
                };
            }
            Err(err) => println!("tbi frozen err :{:?}", err),
        }
    }
    println!("info~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/tbi_info.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("info"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "info",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi info success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("tbi"),
                            String::from("info"),
                        )
                        .unwrap();
                        println!("tbi info :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi info outofe."),
                };
            }
            Err(err) => println!("tbi info err :{:?}", err),
        }
    }
    println!("destroy~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/tbi_destroy.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("tbi"),
            ExecKind::Call,
            String::from("destroy"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "tbi",
            "destroy",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("tbi destroy success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("tbi"),
                            String::from("destroy"),
                        )
                        .unwrap();
                        println!("tbi destroy :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("tbi destroy reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("tbi get outofe."),
                };
            }
            Err(err) => println!("tbi destroy err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
}

fn main_jvm() {
    //account1
    let address1 = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8nft721c01");
    //account2
    let address2 = String::from("nft721ckMueLN92fL1C39nCXtxNR9dnLyD6ypAsto2");
    //contract address
    let contract_address = String::from("nft721ckMueLN92fL1C39nCXtxNR9dnLyD6ypAsto3");

    let geeco = fs::read("./jvm_file_test/GeeCo.class").unwrap();
    let geeco_buf = geeco.as_slice();

    let contract = fs::read("./jvm_file_test/Test.class").unwrap();
    let contract_buf = contract.as_slice();

    //println!("#:{:?}",contract_buf);

    let ctx = Context_JVM::new(
        1,
        ExecKind::Init,
        "Test".to_string(),
        "getheight".to_string(),
        "param: String".to_string(),
        1,
        address1.clone(),
        address1.clone(),
        address1.clone(),
        contract_address.clone(),
        0,
        false,
    );
    let r = match init_jvm(geeco_buf, contract_buf, 1, ctx) {
        Ok(r) => r,
        Err(_) => return,
    };
    println!("init result: {:?}", r);

    let ctx = Context_JVM::new(
        1,
        ExecKind::Init,
        "Test".to_string(),
        "getheight".to_string(),
        "param: String".to_string(),
        1,
        address1.clone(),
        address1.clone(),
        address1.clone(),
        contract_address.clone(),
        0,
        false,
    );
    let r = match call_jvm(geeco_buf, contract_buf, 1, ctx) {
        Ok(r) => r,
        Err(_) => return,
    };
    println!("call result: {:?}", r);
}

fn main_evidence() {
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
    let param = [
        3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
    ];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init(
        "evidence",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000_000,
        true,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: vm_kind.clone(),
    };

    //generate schema and store
    contract.preprocessing(module).unwrap();

    //parameter bytes
    let param = [
        3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
    ];

    //parameter json
    let init_param = fs::read("./wasm_file_test/evidence_init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema: Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(
        &schema,
        &init_param,
        String::from("evidence"),
        ExecKind::Init,
        String::new(),
    )
    .unwrap();

    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init(
        "evidence",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000_000,
        true,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: vm_kind.clone(),
    };

    let ret = contract.exec(Some(module), 0);
    match ret {
        Ok(r) => {
            //println!("test:erc20 ok :{:?}", r);
            match r {
                ContractResult::Success {
                    remaining_energy,
                    actions,
                    event,
                } => println!("evidence init success :{:?}", remaining_energy),
                ContractResult::Data {
                    data,
                    remaining_energy,
                    event,
                } => println!("evidence init data :{:?}", remaining_energy),
                ContractResult::Reject {
                    reason,
                    remaining_energy,
                } => println!("evidence init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy => println!("evidence init outofe."),
            };
        }
        Err(err) => println!("evidence init err :{:?}", err),
    }

    {
        let balance = Amount::from_gtu(0);

        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/evidence_create.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("evidence"),
            ExecKind::Call,
            String::from("create"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("evidence c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("evidence c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("evidence c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("evidence c outofe."),
                };
            }
            Err(err) => println!("evidence c err :{:?}", err),
        }
    }

    {
        let balance = Amount::from_gtu(10000);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_set.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("evidence"),
            ExecKind::Call,
            String::from("set"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("evidence set success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("evidence"),
                            String::from("set"),
                        )
                        .unwrap();
                        println!("evidence set :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("evidence set reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("evidence set outofe."),
                };
            }
            Err(err) => println!("evidence set err :{:?}", err),
        }
    }

    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_get.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("evidence"),
            ExecKind::Call,
            String::from("get"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("evidence get success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("evidence"),
                            String::from("get"),
                        )
                        .unwrap();
                        println!("evidence get :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("evidence get reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("evidence get outofe."),
                };
            }
            Err(err) => println!("evidence get err :{:?}", err),
        }
    }

    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/evidence_info.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("evidence"),
            ExecKind::Call,
            String::from("info"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("evidence info success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("evidence"),
                            String::from("info"),
                        )
                        .unwrap();
                        println!("evidence info :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("evidence info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("evidence info outofe."),
                };
            }
            Err(err) => println!("evidence info err :{:?}", err),
        }
    }
}

fn mainnft() {
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    //wasm
    let modules = fs::read("./wasm_file_test/nft.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8nft721c01");
    //account2
    let address2 = String::from("nft721ckMueLN92fL1C39nCXtxNR9dnLyD6ypAsto2");
    //contract address
    let contract_address = String::from("nft721ckMueLN92fL1C39nCXtxNR9dnLyD6ypAsto3");

    //parameter bytes
    let param = [
        3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
    ];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init(
        "nft",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000_000,
        true,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: VMKind::GeeCo,
    };

    //generate schema and store
    contract.preprocessing(module).unwrap();

    //parameter json
    let init_param = fs::read("./wasm_file_test/nft_init.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema: Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let param = from_json_contract(
        &schema,
        &init_param,
        String::from("nft"),
        ExecKind::Init,
        String::new(),
    )
    .unwrap();

    let balance = Amount::from_gtu(0);
    let init_ctx = Context::new_init(
        "nft",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000_000,
        true,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: vm_kind.clone(),
    }; //GeeCo

    let ret = contract.exec(Some(module), 0);
    match ret {
        Ok(r) => {
            match r {
                ContractResult::Success {
                    remaining_energy,
                    actions,
                    event,
                } => println!("nft init success :{:?}", remaining_energy),
                ContractResult::Data {
                    data,
                    remaining_energy,
                    event,
                } => println!("nft init data :{:?}", remaining_energy),
                ContractResult::Reject {
                    reason,
                    remaining_energy,
                } => println!("nft init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy => println!("nft init outofe."),
            };
        }
        Err(err) => println!("nft init err :{:?}", err),
    }
    println!("new nft ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //new nft~~~~~~~~~~~~
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_new.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("newNFT"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };
        println!("bytes ~~~~~~~~{:?}", param);
        let jstring = to_json_contract(
            &schema,
            &param,
            String::from("nft"),
            ExecKind::Call,
            String::from("newNFT"),
        )
        .unwrap();
        println!("json string ~~~~~~~~{:?}", jstring);
        let p_r = from_json_contract(
            &schema,
            &jstring.as_bytes(),
            String::from("nft"),
            ExecKind::Call,
            String::from("newNFT"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };
        let sender = Address::Contract(ContractAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "newNFT",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft newNFT success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("newNFT"),
                        )
                        .unwrap();
                        println!("nft newNFT :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft newNFT reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft newNFT outofe."),
                };
            }
            Err(err) => println!("nft newNFT err :{:?}", err),
        }
    }
    {
        //new nft~~~~~~~~~~~~
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_new2.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("newNFT"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Contract(ContractAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "newNFT",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft newNFT success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("newNFT"),
                        )
                        .unwrap();
                        println!("nft newNFT :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft newNFT reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft newNFT outofe."),
                };
            }
            Err(err) => println!("nft newNFT err :{:?}", err),
        }
    }
    {
        //new nft~~~~~~~~~~~~
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_new3.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("newNFT"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Contract(ContractAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "newNFT",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft newNFT success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("newNFT"),
                        )
                        .unwrap();
                        println!("nft newNFT :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft newNFT reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft newNFT outofe."),
                };
            }
            Err(err) => println!("nft newNFT err :{:?}", err),
        }
    }
    println!("mint~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_mint.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("mint"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "mint",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft mint success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("mint"),
                        )
                        .unwrap();
                        println!("nft mint :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft mint outofe."),
                };
            }
            Err(err) => println!("nft mint err :{:?}", err),
        }
    }
    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_mint2.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("mint"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "mint",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft mint success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("mint"),
                        )
                        .unwrap();
                        println!("nft mint :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft mint outofe."),
                };
            }
            Err(err) => println!("nft mint err :{:?}", err),
        }
    }
    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_mint3.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("mint"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "mint",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft mint success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("mint"),
                        )
                        .unwrap();
                        println!("nft mint :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft mint outofe."),
                };
            }
            Err(err) => println!("nft mint err :{:?}", err),
        }
    }
    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_mint4.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("mint"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "mint",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft mint success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("mint"),
                        )
                        .unwrap();
                        println!("nft mint :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft mint outofe."),
                };
            }
            Err(err) => println!("nft mint err :{:?}", err),
        }
    }
    {
        //mint~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_mint5.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("mint"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "mint",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft mint success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("mint"),
                        )
                        .unwrap();
                        println!("nft mint :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft mint reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft mint outofe."),
                };
            }
            Err(err) => println!("nft mint err :{:?}", err),
        }
    }
    println!("ownerof ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //ownerof~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_ownerof.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("ownerOf"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Contract(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "ownerOf",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft ownerOf success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("ownerOf"),
                        )
                        .unwrap();
                        println!("nft ownerOf :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft ownerOf reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft ownerOf outofe."),
                };
            }
            Err(err) => println!("nft ownerOf err :{:?}", err),
        }
    }
    println!("transferfrom~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //transferFrom~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_transferfrom.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("transfer"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "transfer",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("nft"));
                        println!("event:{:?}", ret);
                        println!("nft transferFrom success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("nft transferFrom data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!(
                        "nft transferFrom reject :{:?}--{:?}",
                        reason, remaining_energy
                    ),
                    ContractResult::OutOfEnergy => println!("nft transferFrom outofe."),
                };
            }
            Err(err) => println!("nft transferFrom err :{:?}", err),
        }
    }
    println!("ownerof~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //ownerof~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_ownerof2.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("ownerOf"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Contract(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "ownerOf",
            &param,
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft ownerOf success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("ownerOf"),
                        )
                        .unwrap();
                        println!("nft ownerOf :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft ownerOf reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft ownerOf outofe."),
                };
            }
            Err(err) => println!("nft ownerOf err :{:?}", err),
        }
    }
    println!("healthy~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //healthy~~~~~~~~~~~~
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/nft_healthy.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("nft"),
            ExecKind::Call,
            String::from("healthy"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("param error:{:?}", e);
                return;
            }
        };

        let sender = Address::Contract(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

        let call_ctx = Context::new_call(
            "nft",
            "healthy",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        }; //GeeCo
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("nft healthy success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //println!("nft receive data :{:?}", data);
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("nft"),
                            String::from("healthy"),
                        )
                        .unwrap();
                        println!("nft healthy :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("nft healthy reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("nft healthy outofe."),
                };
            }
            Err(err) => println!("nft healthy err :{:?}", err),
        }
    }
}

fn main_geeco_erc20() {
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let gas = false;

    let modules = fs::read("./wasm_file_test/erc20.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8erc200001");
    //account2
    let address2 = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8erc200002");
    //contract address
    let contract_address = String::from("0xdce0c7cb2b3265fbc1ee2d5092bcbb8erc200003");

    //parameter bytes
    let param = [
        3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0,
    ];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init(
        "erc20",
        AccountAddress::from(address1.clone()),
        &param,
        balance,
        AccountAddress::from(contract_address.clone()),
        1000,
        gas,
    );
    let mut contract = Executor {
        db: StorageInstanceRef.write().account_db(),
        context: init_ctx,
        contractkind: ContractKind::Concordium,
        vm_kind: vm_kind.clone(),
    };

    //generate schema and store
    //let ret = contract.preprocessing(module).unwrap();
    match contract.preprocessing(module) {
        Ok(r) => r,
        Err(e) => println!("erc20 preprocessing err :{:?}", e),
    }
    if gas {
        let module = match contract.preprocessing_gas(module) {
            Ok(r) => r,
            Err(e) => return println!("erc20 preprocessing gas  err :{:?}", e),
        };
        let module = module.as_slice();
    }

    //parameter bytes
    {
        println!("init~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        //parameter json
        let init_param = fs::read("./wasm_file_test/init.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let start = Instant::now();
        let param = match from_json_contract(
            &schema,
            &init_param,
            String::from("erc20"),
            ExecKind::Init,
            String::new(),
        ) {
            Ok(r) => r,
            Err(e) => return println!("erc20 from_json_contract err :{:?}", e),
        };

        let balance = Amount::from_gtu(0);
        let init_ctx = Context::new_init(
            "erc20",
            AccountAddress::from(address1.clone()),
            &param,
            balance,
            AccountAddress::from(contract_address.clone()),
            1000,
            gas,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        let ret = contract.exec(Some(module), 0);
        match ret {
            Ok(r) => {
                //println!("test:erc20 ok :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        let ret = to_json_event(&schema, event, String::from("erc20"));
                        println!("event:{:?}", ret);
                        println!("erc20 init success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("erc20 init data :{:?}", remaining_energy),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("erc20 init reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc20 init outofe."),
                };
            }
            Err(err) => println!("erc20 init err :{:?}", err),
        }
        println!("time cost init:{:?} ms", start.elapsed().as_millis());
    }

    {
        let balance = Amount::from_gtu(0);
        println!("receive~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        //parameter bytes
        let param = [
            0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67,
            51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65,
            115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0,
        ];
        //parameter json
        let transfer_param = fs::read("./wasm_file_test/transfer.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let start = Instant::now();
        let p_r = from_json_contract(
            &schema,
            &transfer_param,
            String::from("erc20"),
            ExecKind::Call,
            String::from("receive"),
        );
        let param: Vec<u8> = match p_r {
            Ok(p) => p,
            Err(e) => {
                println!("R:from_json_contract:{:?}", e);
                return;
            }
        };

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            gas,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => {
                        // for log in event.logs{
                        //     to_json_event(&schema, &log,  String::from("erc20"));
                        // }
                        let ret = to_json_event(&schema, event, String::from("erc20"));
                        println!("event:{:?}", ret);
                        println!("erc20 receive success :{:?}", remaining_energy)
                    }
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("erc20 receive data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("erc20 receive reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc20 receive outofe."),
                };
            }
            Err(err) => println!("erc20 receive err :{:?}", err),
        }
        println!("time cost receive:{:?} ms", start.elapsed().as_millis());
    }

    {
        println!("setdata~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];

        //parameter json
        let balance_param = fs::read("./wasm_file_test/erc20_setdata.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let start = Instant::now();
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("erc20"),
            ExecKind::Call,
            String::from("setdata"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            gas,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("erc20 setdata success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        println!("erc20 setdata data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("erc20"),
                            String::from("setdata"),
                        )
                        .unwrap();
                        println!("erc20 setdata :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("erc20 setdata reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc20 setdata outofe."),
                };
            }
            Err(err) => println!("erc20 setdata err :{:?}", err),
        }
        println!("time cost setddata:{:?} ms", start.elapsed().as_millis());
    }

    {
        let balance = Amount::from_gtu(0);
        //parameter bytes
        let param = [
            52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51,
            57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115,
            66, 81, 116, 111, 49,
        ];
        println!("getdata~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        //parameter json
        let balance_param = fs::read("./wasm_file_test/erc20_getdata.json").unwrap();
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let start = Instant::now();
        let param = from_json_contract(
            &schema,
            &balance_param,
            String::from("erc20"),
            ExecKind::Call,
            String::from("getdata"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();

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
            gas,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("erc20 getdata success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        println!("erc20 getdata data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("erc20"),
                            String::from("getdata"),
                        )
                        .unwrap();
                        println!("erc20 getdata :{:#?}", ret);
                        println!("erc20 gas :{:?}", remaining_energy);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("erc20 getdata reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc20 getdata out of energy."),
                };
            }
            Err(err) => println!("erc20 getdata err :{:?}", err),
        }
        println!("time cost getdata:{:?} ms", start.elapsed().as_millis());
    }

    {
        println!("info~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        //parameter json
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);
        let start = Instant::now();
        let param = from_json_contract(
            &schema,
            &[],
            String::from("erc20"),
            ExecKind::Call,
            String::from("info"),
        )
        .unwrap();

        let sender = Address::Account(AccountAddress::from(address1.clone()));
        let invoker = AccountAddress::from(address1.clone());
        let owner = AccountAddress::from(address1.clone());
        let state: Vec<u8> = Vec::new();
        let balance = Amount::from_gtu(0);

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
            gas,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("erc20 info success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("erc20"),
                            String::from("info"),
                        )
                        .unwrap();
                        println!("erc20 info :{:?}____gas:{:?}", ret, remaining_energy);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("erc20 info reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("erc20 info outofe."),
                };
            }
            Err(err) => println!("erc20 info err :{:?}", err),
        }
        println!("time cost info:{:?} ms", start.elapsed().as_millis());
    }
}

fn main_fib() {
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let modules = fs::read("./wasm_file_test/fib.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000002");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000003");

    {
        //parameter bytes
        let balance = Amount::from_gtu(0);
        let init_ctx = Context::new_init(
            "fib",
            AccountAddress::from(address1.clone()),
            &[],
            Amount::from_gtu(0),
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        //generate schema and store
        contract.preprocessing(module).unwrap();

        //parameter json
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);

        let init_ctx = Context::new_init(
            "fib",
            AccountAddress::from(address1.clone()),
            &[],
            Amount::from_gtu(0),
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        let ret = contract.exec(Some(module), 0);
        match ret {
            Ok(r) => {
                //println!("test:erc20 ok :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("fib init success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("fib init data :{:?}", remaining_energy),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("fib init reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("fib init outofe."),
                };
            }
            Err(err) => println!("fib init err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);

        //parameter json
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
            "fib",
            "get",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("fib c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("fib c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("fib c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("fib c outofe."),
                };
            }
            Err(err) => println!("fib c err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        //parameter json
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
            "fib",
            "receive_calc_fib",
            &[],
            &state,
            Amount::from_gtu(2),
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 6);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("fib receive_calc_fib success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => {
                        //old data;
                        //println!("erc20 balance data :{:?}", data);
                        //json data;
                        // let rr:Value = serde_json::from_slice(&data.returndata).unwrap();
                        // println!("erc20 balance data :{:#?}",rr.to_string());

                        //concordium json data
                        let ret = to_json_result(
                            &schema,
                            &data.returndata,
                            String::from("fib"),
                            String::from("receive_calc_fib"),
                        )
                        .unwrap();
                        println!("fib receive_calc_fib :{:?}", ret);
                    }
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!(
                        "fib receive_calc_fib reject :{:?}--{:?}",
                        reason, remaining_energy
                    ),
                    ContractResult::OutOfEnergy => println!("fib receive_calc_fib outofe."),
                };
            }
            Err(err) => println!("fib receive_calc_fib err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);

        //parameter json
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
            "fib",
            "get",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("fib c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("fib c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("fib c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("fib c outofe."),
                };
            }
            Err(err) => println!("fib c err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
}

fn main_bench() {
    let vm_kind = VMKind::WasmTime; //GeeCo     WasmTime

    let modules = fs::read("./wasm_file_test/vc.wasm").unwrap();
    let module = modules.as_slice();

    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000002");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04fib0000003");

    {
        //parameter bytes
        let balance = Amount::from_gtu(0);
        let init_ctx = Context::new_init(
            "fib",
            AccountAddress::from(address1.clone()),
            &[],
            Amount::from_gtu(0),
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        //generate schema and store
        contract.preprocessing(module).unwrap();

        //parameter json
        let db = StorageInstanceRef.write().account_db();
        let mut schema_addr = contract_address.clone().into_bytes();
        schema_addr.insert(42, 66);
        let mut schema: Vec<u8> = Vec::new();
        db.lock().get_bytes(&schema_addr, &mut schema);

        let init_ctx = Context::new_init(
            "vc",
            AccountAddress::from(address1.clone()),
            &[],
            Amount::from_gtu(0),
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: init_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };

        let ret = contract.exec(Some(module), 0);
        match ret {
            Ok(r) => {
                //println!("test:erc20 ok :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("vc init success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("vc init data :{:?}", remaining_energy),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("vc init reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("vc init outofe."),
                };
            }
            Err(err) => println!("vc init err :{:?}", err),
        }
    }
    println!("call~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);

        //parameter json
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
            "vc",
            "call",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("vc c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("vc c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("vc c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("vc c outofe."),
                };
            }
            Err(err) => println!("vc c err :{:?}", err),
        }
    }
    println!("arith~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let balance = Amount::from_gtu(0);

        //parameter json
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
            "vc",
            "arith",
            &[],
            &state,
            balance,
            sender,
            invoker,
            owner,
            AccountAddress::from(contract_address.clone()),
            1000_000,
            true,
        );
        let mut contract = Executor {
            db: StorageInstanceRef.write().account_db(),
            context: call_ctx,
            contractkind: ContractKind::Concordium,
            vm_kind: vm_kind.clone(),
        };
        let ret = contract.exec(None, 0);
        match ret {
            Ok(r) => {
                //println!("erc20 balance :{:?}", r);
                match r {
                    ContractResult::Success {
                        remaining_energy,
                        actions,
                        event,
                    } => println!("vc c success :{:?}", remaining_energy),
                    ContractResult::Data {
                        data,
                        remaining_energy,
                        event,
                    } => println!("vc c data :{:?}", data),
                    ContractResult::Reject {
                        reason,
                        remaining_energy,
                    } => println!("vc c reject :{:?}--{:?}", reason, remaining_energy),
                    ContractResult::OutOfEnergy => println!("vc c outofe."),
                };
            }
            Err(err) => println!("vc c err :{:?}", err),
        }
    }
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
}

use executor::exec::Runtime;
use executor::VM;

pub fn wasmtime_bench(contract: &str, fun: &str) {
    //let modules = compile_modules();
    let module = fs::read("./wasm_file_test/vc.wasm").unwrap();
    let wasm_bytes = module.as_slice();
    let engine = wasmtime::Engine::default();
    let aot_bytes = match engine.precompile_module(wasm_bytes) {
        Ok(b) => b,
        Err(e) => return,
    };

    wasmtime_call(aot_bytes, contract, fun);
}

fn wasmtime_call(modules: Vec<u8>, contract: &str, func: &str) {
    let vm_kind = VMKind::WasmTimeAOT;
    let balance = Amount::from_gtu(0);
    //account1
    let address1 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000001");
    //account2
    let address2 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000002");
    //account3
    let address3 = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000003");
    //contract address
    let contract_address = String::from("0xf6b02a2d47b84e845b7e3623355f04tbi0000009");

    let sender = Address::Account(AccountAddress::from(address1));
    let invoker = AccountAddress::from(address2);
    let owner = AccountAddress::from(address3);
    let state: Vec<u8> = vec![
        48, 120, 102, 54, 98, 48, 50, 97, 50, 100, 52, 55, 98, 56, 52, 101, 56, 52, 53, 98, 55,
        101, 51, 54, 50, 51, 51, 53, 53, 102, 48, 52, 102, 105, 98, 48, 48, 48, 48, 48, 48, 49,
    ];

    let call_ctx = Context::new_call(
        contract,
        func,
        &[],
        &state,
        balance,
        sender,
        invoker,
        owner,
        AccountAddress::from(contract_address),
        1000_000,
        true,
    );
    let mut contract = Runtime {
        context: call_ctx,
        vm_kind: vm_kind,
    };
    let ret = contract.run(&modules, 0);
    match ret {
        Ok(r) => {
            match r {
                ContractResult::Success {
                    remaining_energy,
                    actions,
                    event,
                } => println!("vc c success :{:?}", remaining_energy),
                ContractResult::Data {
                    data,
                    remaining_energy,
                    event,
                } => println!("vc c data :{:?}", data),
                ContractResult::Reject {
                    reason,
                    remaining_energy,
                } => println!("vc c reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy => println!("vc c outofe."),
            };
        }
        Err(err) => println!("vc c err :{:?}", err),
    }
}
