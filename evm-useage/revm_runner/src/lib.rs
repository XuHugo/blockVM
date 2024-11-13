//#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::string::String;
use bytes::Bytes;
use core::str::FromStr;

use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{AccountInfo, Bytecode, ExecutionResult, Output, TransactTo, B160, U256 as rU256, AnalysisKind, B256},
    EVM,
};

//pub const CALCULATOR_EVM_PROGRAM: &str = include_str!("../../bytecode/Calculator.bin-runtime");
pub const CALCULATOR_EVM_PROGRAM: &str = include_str!("../../bytecode/remix0");

pub fn run_revm_calc_contract(input: &str) -> String {
    run_revm(CALCULATOR_EVM_PROGRAM, input)
}

fn run_revm(program: &str, input: &str) -> String {
    // initialise empty in-memory-db
    let mut cache_db = CacheDB::new(EmptyDB::default());
    let pro = "608060405234801561001057600080fd5b50600436106100365760003560e01c80636057361d1461003b5780638f88708b1461006b575b600080fd5b61005560048036038101906100509190610102565b61009b565b604051610062919061013e565b60405180910390f35b61008560048036038101906100809190610102565b6100b1565b604051610092919061013e565b60405180910390f35b60006003826100aa9190610188565b9050919050565b60006004826100c09190610188565b9050919050565b600080fd5b6000819050919050565b6100df816100cc565b81146100ea57600080fd5b50565b6000813590506100fc816100d6565b92915050565b600060208284031215610118576101176100c7565b5b6000610126848285016100ed565b91505092915050565b610138816100cc565b82525050565b6000602082019050610153600083018461012f565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610193826100cc565b915061019e836100cc565b92508282019050808211156101b6576101b5610159565b5b9291505056fea264697066735822122024903d83591253d4529977af96a65708dfc00d841a947c31d9bb75797e90841564736f6c63430008120033";
    let data = hex::decode(program).unwrap();
    let bytecode = Bytes::from(data);
    let code = Bytecode::new_raw(bytecode);

    let be_caller_address = B160::from_str("0xe51a29f538bad980a56129df16ade362fc8b1418").unwrap();
    let be_caller_account = AccountInfo::new(rU256::from(10000000), 0, code);

    cache_db.insert_account_info(be_caller_address, be_caller_account);

    let code2 = Bytecode::default();

    let be_caller_address2 = B160::from_str("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap();
    let be_caller_account2 = AccountInfo::new(rU256::from(10000000), 0, code2);

    cache_db.insert_account_info(be_caller_address2, be_caller_account2);

    // initialise an empty (default) EVM
    let mut evm = EVM::new();

    // insert pre-built database from above
    evm.database(cache_db);

    //cfg
    evm.env.cfg.chain_id = rU256::from(2024);
    evm.env.cfg.spec_id = revm::primitives::MERGE;
    evm.env.cfg.perf_all_precompiles_have_balance = false;
    evm.env.cfg.perf_analyse_created_bytecodes = AnalysisKind::Analyse;

    //block

    evm.env.block.number = rU256::from(99);
    evm.env.block.coinbase = B160::from_str("0x1000000000000000000000000000000000000111").unwrap();
    evm.env.block.timestamp = rU256::from(987654321);

    evm.env.block.prevrandao = Some(B256::from(rU256::from(1)));
    evm.env.block.difficulty = rU256::ZERO;
    evm.env.block.basefee = rU256::ZERO;
    evm.env.block.gas_limit = rU256::MAX;

    // fill in missing bits of env struc
    // change that to whatever caller you want to be
    //evm.env.tx.caller = B160::from_str("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap();
    // account you want to transact with
    evm.env.tx.transact_to = TransactTo::Call(be_caller_address);
    // calldata formed via abigen
    evm.env.tx.data = Bytes::from(hex::decode(input).unwrap());
    // transaction value in wei
    //evm.env.tx.value = rU256::try_from(0).unwrap();


    evm.env.tx.caller = B160::from_str("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap();
    evm.env.tx.gas_limit = 800000;
    evm.env.tx.gas_price = rU256::from(1);
    evm.env.tx.gas_priority_fee = None;
    evm.env.tx.value = rU256::from(0);
    evm.env.tx.chain_id = Some(2024);
    evm.env.tx.nonce = Some(0);
    evm.env.tx.access_list.clear();

    // execute transaction without writing to the DB
    let ref_tx = evm.transact_ref().unwrap();
    // select ExecutionResult struct
    let result = ref_tx.result;

    // unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success { output, reason,  gas_used, gas_refunded,logs } => match output {
            
            Output::Call(value) =>{
                println!("call ^^^^^^^^^^^^{:?}--{:?},",reason,value);
                Some(value)
            } ,
            Output::Create(v, c)=> {
                println!("create~~~~~~~~~~~~~~~~~~~~~~~{:?}--{:?}",v,c);
                None
            },
        },
        ExecutionResult::Revert{ gas_used, output } => {
            println!("Revert~~~~~~~~~~~~~~~~~~~~~~~{}~{:?}",gas_used,output);
            None
        },
        ExecutionResult::Halt{ reason, gas_used } => {
            println!("Halt~~~~~~~~~~~~~~~~~~~~~~~{:?}~{}",reason, gas_used);
            None
        },
    };

    let out = hex::encode(value.unwrap());
    println!("===-------------{}",out);
    out
}


fn init_revm(input: &str) -> String {
    // initialise empty in-memory-db
    let program = CALCULATOR_EVM_PROGRAM;
    let mut cache_db = CacheDB::new(EmptyDB::default());

    let data = hex::decode(program).unwrap();
    let bytecode = Bytes::from(data);
    let code = Bytecode::new_raw(bytecode);
    let code2 = Bytecode::default();

    let be_caller_address = B160::from_str("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap();
    let be_caller_account = AccountInfo::new(rU256::from(10000000), 0, code2);

    cache_db.insert_account_info(be_caller_address, be_caller_account);

    // initialise an empty (default) EVM
    let mut evm = EVM::new();

    // insert pre-built database from above
    evm.database(cache_db);

    //cfg
    evm.env.cfg.chain_id = rU256::from(2024);
    evm.env.cfg.spec_id = revm::primitives::MERGE;
    evm.env.cfg.perf_all_precompiles_have_balance = false;
    evm.env.cfg.perf_analyse_created_bytecodes = AnalysisKind::Analyse;
    //evm.env.cfg.limit_contract_code_size = Some(0x6000);

    //block

    evm.env.block.number = rU256::from(99);
    evm.env.block.coinbase = B160::from_str("0x1000000000000000000000000000000000000111").unwrap();
    evm.env.block.timestamp = rU256::from(987654321);

    evm.env.block.prevrandao = Some(B256::from(rU256::from(1)));
    evm.env.block.difficulty = rU256::ZERO;
    evm.env.block.basefee = rU256::ZERO;
    evm.env.block.gas_limit = rU256::MAX;

    // fill in missing bits of env struc
    // change that to whatever caller you want to be
    //evm.env.tx.caller = B160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    // account you want to transact with
    //evm.env.tx.transact_to = TransactTo::create();
    // calldata formed via abigen
    let c = hex::decode(input).unwrap();
    //println!("{:?}, {}",c,c.len());
    //evm.env.tx.data = Bytes::from(c);
    // transaction value in wei
    //evm.env.tx.value = rU256::try_from(0).unwrap();


    evm.env.tx.caller = B160::from_str("0xf6b02a2d47b84e845b7e3623355f041bcb36daf1").unwrap();
    evm.env.tx.gas_limit = 800000;
    evm.env.tx.gas_price = rU256::from(1);
    evm.env.tx.gas_priority_fee = None;
    evm.env.tx.transact_to = TransactTo::create();
    evm.env.tx.value = rU256::from(0);
    evm.env.tx.data = Bytes::from(c);
    evm.env.tx.chain_id = Some(2024);
    evm.env.tx.nonce = Some(0);
    evm.env.tx.access_list.clear();


    // execute transaction without writing to the DB
    let ref_tx = evm.transact_ref().unwrap();
    // select ExecutionResult struct
    let result = ref_tx.result;

    println!("state:{:#?}",ref_tx.state);

    // unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success { output, reason,.. } => match output {
            
            Output::Call(value) =>{
                println!("^^^^^^^^^^^^{:?}",reason);
                println!("ok~~~~~~~~~~~~~~~~~~~~~~~");
                Some(value)
            } ,
            Output::Create(o, a) => {
                println!("create~~~~~~~~~~~~~~~~~~~~~~~{:?}--{:?}",o, a);
                println!("reaseon:{:?}",reason);
                Some(o)
            },
        },
        ExecutionResult::Revert{ gas_used, output } => {
            println!("Revert~~~~~~~~~~~~~~~~~~~~~~~{}~{:?}",gas_used,output);
            None
        },
        ExecutionResult::Halt{ reason, gas_used } => {
            println!("Halt~~~~~~~~~~~~~~~~~~~~~~~{:?}~{}",reason, gas_used);
            None
        },
    };

    let out = hex::encode(value.unwrap());
    println!("===-------------{}",out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::primitives::U256;

    //#[test]
    fn evm_fibb_works() {
        let data = "61047ff4000000000000000000000000000000000000000000000000000000000000000a";
        let result = run_revm_calc_contract(data);
        assert_eq!(
            result,
            "0000000000000000000000000000000000000000000000000000000000000037"
        );
    }

    #[test]
    fn evm_calc_works() {
        // input Calculator.add(7, 2)
        //0x        771602f700000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000002
        //0x8f88708b0000000000000000000000000000000000000000000000000000000000000005
        let data = "6057361d0000000000000000000000000000000000000000000000000000000000000003";
        //let data = "6057361d000000000000000000000000000000000000000000000000000000000000007b";
        let result = run_revm_calc_contract(data);
        assert_eq!(
            result,
            "0000000000000000000000000000000000000000000000000000000000000006"
        );

    }

    #[test]
    fn evm_init_works() {
        // input Calculator.add(7, 2)
        //0x        771602f700000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000002
        //0x8f88708b0000000000000000000000000000000000000000000000000000000000000005
        let data = "a679109d0000000000000000000000000000000000000000000000000000000000000005";
        //let data = "6057361d000000000000000000000000000000000000000000000000000000000000007b";
        let code = "608060405234801561001057600080fd5b506101f2806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80636057361d1461003b5780638f88708b1461006b575b600080fd5b61005560048036038101906100509190610102565b61009b565b604051610062919061013e565b60405180910390f35b61008560048036038101906100809190610102565b6100b1565b604051610092919061013e565b60405180910390f35b60006003826100aa9190610188565b9050919050565b60006004826100c09190610188565b9050919050565b600080fd5b6000819050919050565b6100df816100cc565b81146100ea57600080fd5b50565b6000813590506100fc816100d6565b92915050565b600060208284031215610118576101176100c7565b5b6000610126848285016100ed565b91505092915050565b610138816100cc565b82525050565b6000602082019050610153600083018461012f565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610193826100cc565b915061019e836100cc565b92508282019050808211156101b6576101b5610159565b5b9291505056fea264697066735822122024903d83591253d4529977af96a65708dfc00d841a947c31d9bb75797e90841564736f6c63430008120033";
        let result = init_revm(code);

    }

    fn vec_2_array<T, const N: usize>(slice: &[T]) -> [T; N]
    where
        T: Copy,
    {
        match slice.try_into() {
            Ok(ba) => ba,
            Err(_) => panic!("Expected a Vec of length {} but it was {}", N, slice.len()),
        }
    }

    #[test]
    fn U256Test() {
        let u1 = U256::from(10000000);
        // let v1 = u1.as_le_bytes();
        let v1 = u1.to_le_bytes_vec();
        println!("v1:{:#?}", v1);

        let u2 = U256::try_from_le_slice(&v1).unwrap();
        assert_eq!(u1, u2);
    }
}
