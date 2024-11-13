use std::{rc::Rc, str::FromStr};

use aptos_api_types::{Bytecode, MoveValue, MoveType};
use aptos_block_executor::task::Transaction;
use aptos_state_view::StateView;
use aptos_vm::{AptosVM, data_cache::IntoMoveResolver};
use geecomove::{
    geecomove::{GeecoMove },
    common, common_ext::tool::MemberId, account::Account, move_tools,
    common_transactions::create_account_txn2,
};
use aptos_framework::natives::code::{PackageRegistry, UpgradePolicy};
use move_core_types::{parser::parse_struct_tag, language_storage::{ModuleId, TypeTag}, identifier::Identifier};
use aptos_types::{on_chain_config::FeatureFlag, transaction::TransactionStatus};
use aptos_types::account_address::{ AccountAddress};
use move_resource_viewer::MoveValueAnnotator;
use serde::{Deserialize, Serialize};
use aptos_types::account_config::AccountResource;

use aptos_types::{
    transaction::{
         ModuleBundle,
         SignedTransaction
    },

};

use move_binary_format::{
    access::ModuleAccess,
    compatibility::Compatibility,
    errors::{verification_error, Location, PartialVMError, VMError, VMResult},
    CompiledModule, IndexKind, file_format::{FunctionDefinition, SignatureToken},
};
//use move_resource_viewer::FatType;
use geecomove::move_tools::ArgWithType;


#[derive(Serialize, Deserialize,Debug)]
struct Item {
    value:u64
}

#[derive(Serialize, Deserialize,Debug)]
struct Ii {
    value:u64
}

#[derive(Serialize, Deserialize,Debug)]
struct Collections {
    items: Vec<Item>
}

fn main(){
   
}

#[test]
fn run_block(){
    use std::sync::{Arc, Mutex};
    use std::thread;
    let shared_counter = Arc::new(Mutex::new(0));
    let mut threads = vec![];

    for i in 0..10 {
        let counter = shared_counter.clone();
        let t = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += i;
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
        });
        threads.push(t);
    }

    for t in threads {
        t.join().unwrap();
    }

    let final_counter = shared_counter.lock().unwrap();
    println!("Final value: {}", *final_counter);
}

#[test]
fn main_co_user() {
    println!("~~~~~geeco move  test~~~~~");
    let total = 10;
    let block_num =2;
    let mut gmove = GeecoMove::new_with_features(vec![FeatureFlag::CODE_DEPENDENCY_CHECK], vec![]);
    //println!("~~~~~create account~~~~~");
    let acc = gmove.new_account_at(AccountAddress::from_hex_literal("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap());
    //println!("~~~~~create init tx~~~~~");
    let txn = gmove.create_publish_package(
        &acc, 
        &common::test_dir_path("gxq"), 
        None,
        |_| {});

    //println!("~~~~~run init contract~~~~~");
    let txstatus = gmove.run(txn);

    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("init {:?}",k);
        },
        _=>{ 
            print!("init error!");
        },

    };
    println!("========================call contract entry func1:create resource============================================================");
    
    let mut txns_create_resource : Vec<SignedTransaction> = Vec::new();
    let txn_tmp = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::start_collection").unwrap(),
        vec![],
        vec![]);
    txns_create_resource.push(txn_tmp);
    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_create_resource);
    println!("create resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                println!("create resource {:?}",k);
            },
            _=>{ 
                print!("create resource error!");
            },
    
        };
    };
    
    for c in 1..block_num{
        println!("=========================call contract entry func2:operate resource{}===========================================================",c);
        let mut i = 0;
        let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
        for j in 1..total{
            let txn_tmp = gmove.create_entry_function(        
                &acc,
                str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
                vec![],
                vec![]);
    
                txns_operate_resource.push(txn_tmp);
        }
    
        let start = std::time::Instant::now();
        let resource_tx_status = gmove.run_block(txns_operate_resource);
        println!("operate resource: {} ms",start.elapsed().as_millis());
        for txstatus in resource_tx_status.iter(){
            match txstatus{
                TransactionStatus::Keep(k)=>{
                    //println!("operate resource {:?}",k);
                    ()
                },
                _=>{ 
                    //print!("operate resource error!");
                    i=i+1;
                },
        
            };
        };
        println!("error:{}",i);
    }
    println!("================================call contract  read resource====================================================");
    let get = false;
    if get{
        {
            let collections = gmove
            .read_resource::<Collections>(
                acc.address(),
                parse_struct_tag("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::Collections").unwrap(),
            )
            .unwrap();
            println!("{:?}:{:#?}",acc.address().to_hex_literal(), collections.items);
    
            let MemberId {
                module_id,
                member_id: function_id,
            } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::get2").unwrap();
        
            let args = vec![
                ArgWithType::from_str(&format!("address:{}",acc.address().to_hex_literal())).unwrap(),
            ];
    
            let args_vec: Vec<Vec<u8>> = args
            .into_iter()
            .map(|arg_with_type| arg_with_type.arg)
            .collect();
    
            get_resource_move(    
                &gmove.executor.data_store,
                module_id,
                function_id,
                vec![],
                args_vec
            );
    
        }
    }
    
   
}


#[test]
fn main_co_uses() {
    println!("~~~~~geeco move  test~~~~~");
    let mut gmove = GeecoMove::new_with_features(vec![FeatureFlag::CODE_DEPENDENCY_CHECK], vec![]);
    //println!("~~~~~create account~~~~~");
    let acc = gmove.new_account_at(AccountAddress::from_hex_literal("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap());
    //println!("~~~~~create init tx~~~~~");
    let txn = gmove.create_publish_package(
        &acc, 
        &common::test_dir_path("gxq"), 
        None,
        |_| {});

    //println!("~~~~~run init contract~~~~~");
    let txstatus = gmove.run(txn);

    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("init {:?}",k);
        },
        _=>{ 
            print!("init error!");
        },

    };
    println!("==================================create account==================================================");
    let mut accounts:Vec<Account> = Vec::new();

    for i in 1000..6000{
        let addr = format!("0x{}", i.to_string());
        let account_tmp = gmove.new_account_at(AccountAddress::from_hex_literal(&addr).unwrap());
        accounts.push(account_tmp);
    }
    println!("========================call contract entry func1:create resource============================================================");
    let mut i = 0;
    
    let mut txns_create_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::start_collection").unwrap(),
            vec![],
            vec![]);

        txns_create_resource.push(txn_tmp);
    }
    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_create_resource);
    println!("create resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("create resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("create resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);

    println!("=========================call contract entry func2:operate resource1===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource2===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource3===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource4===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource5===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource6===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource7===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("=========================call contract entry func2:operate resource8===========================================================");
    let mut i = 0;
    let mut txns_operate_resource : Vec<SignedTransaction> = Vec::new();
    for addr in accounts.iter(){
        let txn_tmp = gmove.create_entry_function(        
            addr,
            str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
            vec![],
            vec![]);

            txns_operate_resource.push(txn_tmp);
    }

    let start = std::time::Instant::now();
    let resource_tx_status = gmove.run_block(txns_operate_resource);
    println!("operate resource: {} ms",start.elapsed().as_millis());
    for txstatus in resource_tx_status.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                //println!("operate resource {:?}",k);
                ()
            },
            _=>{ 
                //print!("operate resource error!");
                i=i+1;
            },
    
        };
    };
    println!("error:{}",i);
    println!("================================call contract  read resource====================================================");
    let get = false;
    if get{
        for addr in accounts.iter(){
            let collections = gmove
            .read_resource::<Collections>(
                addr.address(),
                parse_struct_tag("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::Collections").unwrap(),
            )
            .unwrap();
            println!("{:?}:{:#?}",addr.address().to_hex_literal(), collections.items);
    
            let MemberId {
                module_id,
                member_id: function_id,
            } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::get2").unwrap();
        
            let args = vec![
                ArgWithType::from_str(&format!("address:{}",addr.address().to_hex_literal())).unwrap(),
            ];
    
            let args_vec: Vec<Vec<u8>> = args
            .into_iter()
            .map(|arg_with_type| arg_with_type.arg)
            .collect();
    
            get_resource_move(    
                &gmove.executor.data_store,
                module_id,
                function_id,
                vec![],
                args_vec
            );
    
        }
    }
    
   
}

#[test]
fn compile_module() {
    println!("~~~~~geeco move  test~~~~~");
    let mut gmove = GeecoMove::new_with_features(vec![FeatureFlag::CODE_DEPENDENCY_CHECK], vec![]);
    println!("~~~~~create account~~~~~");
    let acc = gmove.new_account_at(AccountAddress::from_hex_literal("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap());
    println!("~~~~~create init tx~~~~~");
    let txn = gmove.create_publish_package(
        &acc, 
        &common::test_dir_path("gxq"), 
        None,
        |_| {});

    println!("~~~~~run init contract~~~~~");
    let txstatus = gmove.run(txn);

    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("init {:?}",k);
        },
        _=>{ 
            print!("init error!");
        },

    };

    

    let args = vec![
            ArgWithType::from_str(&format!("u8:11")).unwrap(),
            ArgWithType::from_str(&format!("u8:12")).unwrap(),
        ];

    let args_vec: Vec<Vec<u8>> = args
        .into_iter()
        .map(|arg_with_type| arg_with_type.arg)
        .collect();
    println!("~~~~~call contract entry func3:destory resource~~~~~");
    let txn3 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::MoveTest::rem_u8").unwrap(),
        vec![],
        args_vec);
    let txstatus = gmove.run(txn3);
    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("call destory resource: {:?}",k);
        },
        _=>{ 
            print!("call destory resource: error!");
        },

    };


    println!("~~~~~call contract entry func4:destory resource~~~~~");
    let txn3 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::MoveTest::permissioned").unwrap(),
        vec![],
        vec![]);
    let txstatus = gmove.run(txn3);
    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("call destory resource: {:?}",k);
        },
        _=>{ 
            print!("call destory resource: error!");
        },

    };

    let args = vec![
        ArgWithType::from_str(&format!("address:0xf6b02a2d47b84e845b7e3623355f041bcb36daf1")).unwrap(),
    ];

    let MemberId {
        module_id,
        member_id: function_id,
    } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::MoveTest::get_result").unwrap();

    let args_vec: Vec<Vec<u8>> = args
    .into_iter()
    .map(|arg_with_type| arg_with_type.arg)
    .collect();

    let mut parsed_type_args = Vec::new();
        parsed_type_args.push(
            MoveType::from_str("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::StandardErc21::CoinStore").unwrap(),
        );
    let mut type_args: Vec<TypeTag> = Vec::new();
    // These TypeArgs are used for generics
    // for type_arg in parsed_type_args.into_iter() {
    //     let type_tag = TypeTag::try_from(type_arg).unwrap();
    //     type_args.push(type_tag)
    // }
    type_args.push(TypeTag::U8);

    get_resource_move(    
        &gmove.executor.data_store,
        module_id,
        function_id,
        type_args,
        args_vec
    );
}

#[test]
fn main2() {
    println!("~~~~~geeco move  test~~~~~");
    let mut gmove = GeecoMove::new_with_features(vec![FeatureFlag::CODE_DEPENDENCY_CHECK], vec![]);
    println!("~~~~~create account~~~~~");
    let acc = gmove.new_account_at(AccountAddress::from_hex_literal("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap());
    println!("~~~~~create init tx~~~~~");
    let txn = gmove.create_publish_package(
        &acc, 
        &common::test_dir_path("gxq"), 
        None,
        |_| {});

    println!("~~~~~run init contract~~~~~");
    let txstatus = gmove.run(txn);

    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("init {:?}",k);
        },
        _=>{ 
            print!("init error!");
        },

    };
    println!("====================================================================================");
    println!("~~~~~create account tx~~~~~");
    let new_addr = AccountAddress::from_hex_literal("0xf6b02a2d47b84e845b7e3623355f041bcb36daf0").unwrap();

    let txn = create_account_txn2(
        &acc, 
        new_addr, 
        11);

    let txstatus = gmove.run(txn);

    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("create account tx {:?}",k);
        },
        TransactionStatus::Discard(e) => {
            println!("create account tx Discard! {:?}",e);
        },
        TransactionStatus::Retry => {
            println!("create account tx Retry!");
        },
    };

    println!("====================================================================================");
    println!("~~~~~create account~~~~~");
    let mut accounts:Vec<Account> = Vec::new();

    for i in 100..110{
        let addr = format!("0x{}", i.to_string());
        let account_tmp = gmove.new_account_at(AccountAddress::from_hex_literal(&addr).unwrap());
        accounts.push(account_tmp);
    }
    println!("====================================================================================");
    println!("~~~~~create call tx~~~~~");

    let txn1 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::start_collection").unwrap(),
        vec![],
        vec![]);

    // let txn2 = gmove.create_entry_function(        
    //     &acc,
    //     str::parse("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::Collection::sizes").unwrap(),
    //     vec![],
    //     vec![]);

    let txn3 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
        vec![],
        vec![]);

    // let txn4 = gmove.create_entry_function(        
    //     &acc1,
    //     str::parse("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::Collection::start_collection").unwrap(),
    //     vec![],
    //     vec![]);
    // let txn5 = gmove.create_entry_function(        
    //     &acc2,
    //     str::parse("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::Collection::start_collection").unwrap(),
    //     vec![],
    //     vec![]);
    // let txn6 = gmove.create_entry_function(        
    //     &acc1,
    //     str::parse("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::Collection::add_item").unwrap(),
    //     vec![],
    //     vec![]);
    // let txn7 = gmove.create_entry_function(        
    //     &acc2,
    //     str::parse("0x160b54be617f4bff07bd6c994fc6dd17a69d5e4e::Collection::add_item").unwrap(),
    //     vec![],
    //     vec![]);

    println!("~~~~~call contract entry func1:create resource~~~~~");
    let txstatus = gmove.run(txn1);
    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("call1 {:?}",k);
        },
        _=>{ 
            print!("call1 error!");
        },

    };
    // let txstatus = gmove.run(txn4);
    // match txstatus{
    //     TransactionStatus::Keep(k)=>{
    //         println!("call4 {:?}",k);
    //     },
    //     _=>{ 
    //         print!("call4 error!");
    //     },

    // };
    // let txstatus = gmove.run(txn5);
    // match txstatus{
    //     TransactionStatus::Keep(k)=>{
    //         println!("call5 {:?}",k);
    //     },
    //     _=>{ 
    //         print!("call5 error!");
    //     },

    // };

    println!("~~~~~call contract entry func2:operate resource~~~~~");
    let txns = vec![txn3]; //vec![txn2, txn3, txn6, txn7];
    let txstatus2 = gmove.run_block(txns);
    for txstatus in txstatus2.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                println!("call {:?}",k);
            },
            _=>{ 
                print!("call error!");
            },
    
        };
    };

    println!("~~~~~call contract entry func3:destory resource~~~~~");
    let txn3 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::destory_resource").unwrap(),
        vec![],
        vec![]);
    let txstatus = gmove.run(txn3);
    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("call destory resource: {:?}",k);
        },
        _=>{ 
            print!("call destory resource: error!");
        },

    };

    let args = vec![
        ArgWithType::from_str(&format!("address:0xf6b02a2d47b84e845b7e3623355f041bcb36daf1")).unwrap(),
    ];

    let MemberId {
        module_id,
        member_id: function_id,
    } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::exists_at").unwrap();

    let args_vec: Vec<Vec<u8>> = args
    .into_iter()
    .map(|arg_with_type| arg_with_type.arg)
    .collect();


    get_resource_move(    
        &gmove.executor.data_store,
        module_id,
        function_id,
        vec![],
        args_vec
    );


    // let txn3 = gmove.create_entry_function(        
    //     &acc,
    //     str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::sizes").unwrap(),
    //     vec![],
    //     args_vec);

    // let txns = vec![txn3]; //vec![txn2, txn3, txn6, txn7];
    // let txstatus2 = gmove.run_block(txns);
    // for txstatus in txstatus2.iter(){
    //     match txstatus{
    //         TransactionStatus::Keep(k)=>{
    //             println!("call2 borexists_atrow {:?}",k);
    //         },
    //         _=>{ 
    //             print!("call2 exists_at error!");
    //         },
    
    //     };
    // };


    println!("~~~~~create call tx~~~~~");

    let txn1 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::start_collection").unwrap(),
        vec![],
        vec![]);
    let txstatus = gmove.run(txn1);
    match txstatus{
        TransactionStatus::Keep(k)=>{
            println!("call2 move to {:?}",k);
        },
        _=>{ 
            print!("call2 move to error!");
        },

    };

    let txn3 = gmove.create_entry_function(        
        &acc,
        str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::add_item").unwrap(),
        vec![],
        vec![]);

    let txns = vec![txn3]; //vec![txn2, txn3, txn6, txn7];
    let txstatus2 = gmove.run_block(txns);
    for txstatus in txstatus2.iter(){
        match txstatus{
            TransactionStatus::Keep(k)=>{
                println!("call2 borrow {:?}",k);
            },
            _=>{ 
                print!("call2 borrow error!");
            },
    
        };
    };
    
    println!("~~~~~call contract  read resource~~~~~");
    let collections = gmove
        .read_resource::<Collections>(
            acc.address(),
            parse_struct_tag("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::Collections").unwrap(),
        )
        .unwrap();
    println!("{:#?}",collections.items);

    let MemberId {
        module_id,
        member_id: function_id,
    } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::get3").unwrap();


    get_resource_move(    
        &gmove.executor.data_store,
        module_id,
        function_id,
        vec![],
        vec![]
    );

   
}



fn get_resource_geeco(        
    state_view: &impl StateView,
    module_id: ModuleId,
    func_name: Identifier,
    type_args: Vec<TypeTag>,
    arguments: Vec<Vec<u8>>,
    )-> Vec<Vec<u8>>{
    let return_vals = AptosVM::execute_view_function(
        &state_view,
        module_id,
        func_name,
        type_args,
        arguments,
        2_000_000,
    ).unwrap();
    println!("{:?}",return_vals);
    return_vals
}




// fn deserialize_module_bundle(modules: &ModuleBundle) -> Result<Vec<CompiledModule>,()> {
//     let mut result = vec![];
//     for module_blob in modules.iter() {
//         match CompiledModule::deserialize(module_blob.code()) {
//             Ok(module) => {
//                 result.push(module);
//             }
//             Err(_err) => {
//                 return Err(())
//             }
//         }
//     }
//     Ok(result)
// }


fn get_resource_move(    
    state_view: &impl StateView,
    module_id: ModuleId,
    func_name: Identifier,
    type_args: Vec<TypeTag>,
    arguments: Vec<Vec<u8>>,
){

    let return_vals = match AptosVM::execute_view_function(
        &state_view,
        module_id.clone(),
        func_name.clone(),
        type_args.clone(),
        arguments,
        2_000_000,
    ){
        Ok(o) => o,
        Err(e) => {
            println!("execute error:{}",e.to_string());
            return
        },
    };

    let inner = state_view.into_move_resolver();
    let cover = MoveValueAnnotator::new(&inner);
    
    let code = cover.get_module(&module_id).unwrap() as Rc<dyn Bytecode>;
    let mvfunction =match  code.find_function(&func_name){
        Some(mf)=> mf,
        None=> {
            println!("can't find function!!!");
            return
        },
    };
    let mvreturn = mvfunction.return_;

    let mv_type =  if mvreturn.len() == 1{
        match mvreturn[0]{
            MoveType::GenericTypeParam { index } => {
                
                if type_args.clone().len() <= index.into(){
                    println!("error end");
                    return 
                }else{
                    let id = index as usize;
                    vec![type_args[id].clone()]
                }
            },
            _=>{
                let a = mvreturn.into_iter().map(TypeTag::try_from).collect::<anyhow::Result<Vec<_>>>();
                let mv_type = match a{
                    Ok(o)=>o,
                    Err(e)=> {
                        println!("error end!{:?}",e.to_string());
                        return 
                    },
                };
                mv_type
            }
        }

    }else{
        let a = mvreturn.into_iter().map(TypeTag::try_from).collect::<anyhow::Result<Vec<_>>>();
        let mv_type = match a{
            Ok(o)=>o,
            Err(e)=> {
                println!("error end!{:?}",e.to_string());
                return 
            },
        };
        mv_type
    };
    
    

    // let mv_type = match a{
    //     Ok(o)=>o,
    //     Err(e)=> {
    //         println!("error end!{:?}",e.to_string());
    //         return 
    //     },
    // };

    let ret = return_vals
    .into_iter()
    .zip(mv_type.into_iter())
    .map(
        |(v, ty)|{
            TryInto::<MoveValue>::try_into(cover.view_value(&ty, &v)?)
        }
    )
    .collect::<anyhow::Result<Vec<_>>>();

    match ret{
        Ok(o)=> {
            //println!("{}",o.len());
            //println!("{:?}",o[0].json());
            return
        },
        Err(e)=> return,
    }
}



fn get_r(gmove :GeecoMove){
    let MemberId {
        module_id,
        member_id: function_id,
    } = str::parse("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection::get2").unwrap();

    let return_vals = get_resource_geeco(
        &gmove.executor.data_store, 
        module_id.clone(), 
        function_id.clone(), 
        vec![],
        vec![]);

    // let cm = deserialize_module_bundle(&mb).unwrap();
    // let compliedmodule = Rc::new(cm[0].clone());
    // let code = compliedmodule as Rc<dyn Bytecode>;
    // let mvfunction = match code.find_function(&function_id){
    //     Some(mf)=> mf,
    //     None=> {
    //         println!("can't find function!!!");
    //         return
    //     },
    // };
    // let mvreturn = mvfunction.return_;
    
    // let a = mvreturn.into_iter().map(TypeTag::try_from).collect::<anyhow::Result<Vec<_>>>();

    // let mv_type = match a{
    //     Ok(o)=>o,
    //     Err(e)=> {
    //         println!("error end!");
    //         return 
    //     },
    // };

    let inner = gmove.executor.data_store.into_move_resolver();
    let cover = MoveValueAnnotator::new(&inner);
    
    // pub fn try_into_move_value(typ: &TypeTag, bytes: &[u8]) -> Result<MoveValue> {
    //     self.inner.view_value(typ, bytes)?.try_into()
    // }
    let code = cover.get_module(&module_id).unwrap() as Rc<dyn Bytecode>;
    let mvfunction =match  code.find_function(&function_id){
        Some(mf)=> mf,
        None=> {
            println!("can't find function!!!");
            return
        },
    };
    let mvreturn = mvfunction.return_;
    
    let a = mvreturn.into_iter().map(TypeTag::try_from).collect::<anyhow::Result<Vec<_>>>();

    let mv_type = match a{
        Ok(o)=>o,
        Err(e)=> {
            println!("error end!");
            return 
        },
    };

    let ret = return_vals
    .into_iter()
    .zip(mv_type.into_iter())
    .map(
        |(v, ty)|{
            TryInto::<MoveValue>::try_into(cover.view_value(&ty, &v)?)
        }
    )
    .collect::<anyhow::Result<Vec<_>>>();

    match ret{
        Ok(o)=> {
            println!("{}",o.len());
            println!("{:?}",o[0].json());
        },
        Err(e)=> return,
    }

}