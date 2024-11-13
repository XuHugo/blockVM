use anyhow::{ bail, ensure, anyhow, Result};
use crate::types::Context;
use wasm_chain_integration::*;
use concordium_contracts_common::*;
use std::{
    fs,
    time::Instant,
};

// init contract
pub fn init_wasm(func_name: &str, context: Context, binary: &[u8], amount: i64) -> Result<InitResult> {
    let init_ctx: InitContext<&[u8]> = InitContext {
        // metadata:        ChainMetadata {
        //     slot_time: Timestamp::from_timestamp_millis(0),
        //     height: 2000,
        //     tx_hash: "txhash0xf6b02a2d47b84e845b7e3623355f041bcb36daf1".to_string(),
        // },
        metadata: context.metadata,
        init_origin:     context.owner,
        sender_policies: &[],
    };
    let mut gas = u64::MAX;
    if context.gas{
        gas = context.gas_limit;
    }

    let start = Instant::now();
    let ret = match invoke_init_from_source(
        binary,
        amount as u64,//context.self_balance.micro_gtu,
        init_ctx,
        func_name,
        context.param,
        gas,
    ){
        Ok(ret) => ret,
        Err(e) => {
            println!("geecowasm init =>{:?}", e);
            return Err(e);
        },
    };
    println!("time cost init:{:?} ms",start.elapsed().as_millis());
    Ok(ret)
}

// call contract
pub fn receive_wasm(func_name: &str, context: Context, binary: &[u8], amount: i64) -> Result<ReceiveResult> {
    let receive_ctx: ReceiveContext<&[u8]> = ReceiveContext {
        // metadata:        ChainMetadata {
        //     slot_time: Timestamp::from_timestamp_millis(0),
        //     height: 2000,
        //     tx_hash: "txhash0xf6b02a2d47b84e845b7e3623355f041bcb36daf1".to_string(),
        // },
        metadata: context.metadata,
        invoker: context.invoker,
        self_address: context.self_address,//context.self_address,
        self_balance: context.self_balance,
        sender: context.sender,
        owner:context.owner,
        sender_policies: &[],
    };
    let mut gas = u64::MAX;
    if context.gas{
        gas = context.gas_limit;
    }

    let start = Instant::now();
    let ret = match invoke_receive_from_source(
        binary,
        amount as u64,//context.self_balance.micro_gtu,
        receive_ctx,
        &context.state.state,
        func_name,
        &context.param,
        gas,
    ){
        Ok(ret) => ret,
        Err(e) => {
            println!("geecowasm recv=>{:?}", e);
            return Err(e);
        },
    };
    //println!("time cost recv:{:?} ms",start.elapsed().as_millis());
    Ok(ret)
}