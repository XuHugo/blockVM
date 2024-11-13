use std::{
    fs,
    time::Instant,
};
use concordium_contracts_common::{ Amount, Address, AccountAddress};
use executor::{types::{Context, ContractResult, ContractKind, ExecKind, VMKind}, exec::Executor, Contract, utils::{from_json_contract, to_json_result}};
use storage::{StorageInstanceRef};
use serde_json::{Value};
use wasm_chain_integration::{ Action};

#[test]
fn escrow_run(){
    let vm_kind = VMKind::GeeCo; //GeeCo     WasmTime

    let c = AccountAddress([52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116]);
    let d = String::from(c);
    println!("addr to string:{:?}",d);

    let modules = fs::read("./wasm_file_test/escrow.wasm").unwrap();
    let module = modules.as_slice();

    //buyer
    let address1 = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ1");
    //seller
    let address2 = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ2");
    //arbiter
    let address3 = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ3");
    //contract address
    let contract_address = String::from("4qGpgAhkMueLN92fL1C39nCXtxNR9dnLyD6ypAsBQ4");

    //parameter bytes
    let param = [3, 0, 0, 0, 82, 77, 66, 1, 0, 0, 0, 36, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("escrow", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()),1000_000,true);
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
    let start = Instant::now();
    let param = from_json_contract(&schema, &init_param, String::from("escrow"), ExecKind::Init, String::new()).unwrap();

    let balance = Amount::from_gtu(10000);
    let init_ctx = Context::new_init("escrow", AccountAddress::from(address1.clone()), &param, balance, AccountAddress::from(contract_address.clone()),1000_000,true);
    let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: init_ctx,contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };

    let ret = contract.exec(Some(module), 0);
    match ret{
        Ok(r) => {
            //println!("test:escrow ok :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("escrow init success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> println!("escrow init data :{:?}", remaining_energy),
                ContractResult::Reject{ reason, remaining_energy } => println!("escrow init reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("escrow init outofe."),
            };
        },
        Err(err)=>println!("escrow init err :{:?}", err),
    }
    println!("time cost init:{:?} ms",start.elapsed().as_millis());

    let balance =  Amount::from_gtu(10000);

    //parameter bytes
    let param = [0, 52, 113, 71, 112, 103, 65, 104, 107, 77, 117, 101, 76, 78, 57, 50, 102, 76, 49, 67, 51, 57, 110, 67, 88, 116, 120, 78, 82, 57, 100, 110, 76, 121, 68, 54, 121, 112, 65, 115, 66, 81, 116, 111, 50, 45, 0, 0, 0, 0, 0, 0, 0];
    //submit deposit
    let transfer_param = fs::read("./wasm_file_test/escrow_receive.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);

    let start = Instant::now();
    let param = from_json_contract(&schema, &transfer_param, String::from("escrow"), ExecKind::Call, String::from("receive")).unwrap();

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "escrow",
        "receive",
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
    let ret = contract.exec(None, 105);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions, event }=> println!("escrow receive success :{:?}", remaining_energy),
                ContractResult::Data { data,remaining_energy, event }=> println!("escrow receive data :{:?}", data),
                ContractResult::Reject{ reason, remaining_energy } => println!("escrow receive reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("escrow receive outofe."),
            };
        },
        Err(err)=>println!("escrow receive err :{:?}", err),
    }

    println!("time cost receive:{:?} ms",start.elapsed().as_millis());

    //accept delivery
    let transfer_param = fs::read("./wasm_file_test/escrow_acceptdelivery.json").unwrap();
    let db = StorageInstanceRef.write().account_db();
    let mut schema_addr = contract_address.clone().into_bytes();
    schema_addr.insert(42, 66);
    let mut schema:Vec<u8> = Vec::new();
    db.lock().get_bytes(&schema_addr, &mut schema);
    let start = Instant::now();
    let param = from_json_contract(&schema, &transfer_param, String::from("escrow"), ExecKind::Call, String::from("receive")).unwrap();

    let sender = Address::Account(AccountAddress::from(address1.clone()));
    let invoker = AccountAddress::from(address1.clone());
    let owner = AccountAddress::from(address1.clone());
    let state:Vec<u8> = Vec::new();

    let call_ctx = Context::new_call(
        "escrow",
        "receive",
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
    let ret = contract.exec(None, 105);
    match ret{
        Ok(r) => {
            //println!("erc20 balance :{:?}", r);
            match r{
                ContractResult::Success { remaining_energy, actions , event}=> {
                    println!("escrow receive success :{:?}", remaining_energy);
                    let mut ret:Vec<bool> = Vec::new();
                    for (i, action) in actions.iter().enumerate() {

                        match action {
                            Action::Send {
                                data,
                            } => {
                                let name_str = std::str::from_utf8(&data.name)
                                    .expect("Target name is not a valid UTF8 sequence.");
                                //transfer & contract :xq
                                eprintln!(
                                    "{}: send a message to contract at ({:?}), calling \
                                             method {} with amount {} and parameter{:?}",
                                    i,
                                    data.to_addr,
                                    name_str,
                                    data.amount,
                                    data.parameter,
                                );
                                ret.push(true);
                            }
                            Action::SimpleTransfer {
                                data,
                            } => {
                                eprintln!(
                                    "{}: simple transfer to account {:?} of amount {}",
                                    i,
                                    data.to_addr,
                                    data.amount
                                );

                                ret.push(false);
                            }
                            Action::And {
                                l,
                                r,
                            } => {
                                eprintln!("{}: AND composition of {} and {}", i, l, r);
                                let and = ret[*l as usize] && ret[*r as usize];
                                ret.push(and);
                            },
                            Action::Or {
                                l,
                                r,
                            } => {
                                eprintln!("{}: OR composition of {} and {}", i, l, r);
                                let or = ret[*l as usize] || ret[*r as usize];
                                ret.push(or);
                            },
                            Action::Accept => {
                                eprintln!("{}: Accept", i );

                                ret.push(true);
                            },
                            Action::Get => {
                                println!(": Get");
                            },
                        }

                    }

                    println!("ret:{:?}",ret);
                },
                ContractResult::Data { data,remaining_energy,event }=> println!("escrow receive data :{:?}", data),
                ContractResult::Reject{ reason, remaining_energy } => println!("escrow receive reject :{:?}--{:?}", reason, remaining_energy),
                ContractResult::OutOfEnergy=> println!("escrow receive outofe."),
            };
        },
        Err(err)=>println!("escrow receive err :{:?}", err),
    }
    println!("time cost receive:{:?} ms",start.elapsed().as_millis());
    //
    // let transfer_param = fs::read("./wasm_file_test/escrow_acceptdelivery.json").unwrap();
    // let db = StorageInstanceRef.write().account_db();
    // let mut schema_addr = contract_address.clone().into_bytes();
    // schema_addr.insert(42, 66);
    // let mut schema:Vec<u8> = Vec::new();
    // db.lock().get_bytes(&schema_addr, &mut schema);
    // let param = from_json_contract(&schema, &transfer_param, String::from("escrow"), ExecKind::Call, String::from("receive")).unwrap();
    //
    // let sender = Address::Account(AccountAddress::from(address1.clone()));
    // let invoker = AccountAddress::from(address1.clone());
    // let owner = AccountAddress::from(address1.clone());
    // let state:Vec<u8> = Vec::new();
    //
    // let call_ctx = Context::new_call(
    //     "escrow",
    //     "receive",
    //     &param,
    //     &state,
    //     balance,
    //     sender,
    //     invoker,
    //     owner,
    //     AccountAddress::from(contract_address.clone()),
    // );
    // let mut contract = Executor{ db: StorageInstanceRef.write().account_db(), context: call_ctx, contractkind:ContractKind::Concordium, vm_kind: vm_kind.clone() };
    // let ret = contract.exec(None, 105);
    // match ret{
    //     Ok(r) => {
    //         //println!("erc20 balance :{:?}", r);
    //         match r{
    //             ContractResult::Success { remaining_energy }=> println!("escrow2 receive success :{:?}", remaining_energy),
    //             ContractResult::Data { data,remaining_energy }=> println!("escrow2 receive data :{:?}", data),
    //             ContractResult::Reject{ reason, remaining_energy } => println!("escrow2 receive reject :{:?}--{:?}", reason, remaining_energy),
    //             ContractResult::OutOfEnergy=> println!("escrow2 receive outofe."),
    //         };
    //     },
    //     Err(err)=>println!("escrow2 receive err :{:?}", err),
    // }
}