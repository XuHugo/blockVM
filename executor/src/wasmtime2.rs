use anyhow::{ bail, ensure, anyhow, Result};
use block_executor::task::ModulePath;
use lazy_static::__Deref;
use crate::{
    types::{Context, ContextStm, ADDRESS_SIZE, GAS_SCALE, GAS_ENV_FUNC_BASE, GAS_INIT_FUNC_BASE, WasmTransactionOutput, ExecKind},
};
use wasmtime::*;
use wasmtime::Val::{I64};
use wasm_chain_integration::*;
use concordium_contracts_common::{ Serial};
use std::{
    time::Instant, convert::TryInto, sync::Arc,
};

use aptos_types::{
    account_address::AccountAddress,
    access_path::AccessPath,
    write_set::WriteSet, 
    state_store::state_key::StateKey, 
    write_set::WriteOp
};

pub fn hex_str_to_bytes(s: &str) -> Result<Vec<u8>> {
    for (i, ch) in s.chars().enumerate() {
        if !ch.is_digit(16) {
            bail!(
                "invalid character position {}",
                i
            );
        }
    }

    let input: Vec<_> = s.chars().collect();

    let decoded: Vec<u8> = input
        .chunks(2)
        .map(|chunk| {
            ((chunk[0].to_digit(16).unwrap() << 4) | (chunk[1].to_digit(16).unwrap())) as u8
        })
        .collect();

    Ok(decoded)
}

// Returns the passed Wasm error code if it is negative.
// This function should only be called on negative numbers.
fn reason_from_wasm_error_code(n: i32) -> Result<i32> {
    ensure!(
        n < 0,
        "Wasm return value of {} is treated as an error. Only negative should be treated as error.",
        n
    );
    Ok(n)
}

// env function for contract;
fn host_func_init<K:ModulePath>(engine: &Engine) -> Result<Linker<ContextStm<K>>>
{
    let mut linker = Linker::new(&engine);

    linker.func_wrap("concordium", "get_init_origin",|mut caller: Caller<'_, ContextStm<K>>, ptr: i32| {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let s:&[u8] = &caller.data_mut().origin;
        //println!("read s:{:?}",s);
        let b = s.to_owned();

        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return Err(Trap::new("failed to find get_init_origin memory")),
        };
        let  _data = mem.write(caller, ptr as usize, &b);
        Ok(())
    })?;

    linker.func_wrap("concordium", "write_state",  |mut caller: Caller<'_, ContextStm<K>>, start: i32, len: i32, offset: i32| ->i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -106i32,
        };
        let  data = mem.data(&caller)
            .get(start as u32 as usize..)
            .and_then(|arr| arr.get(..len as u32 as usize));
        let  string = match data {
            Some(s) => s,
            None => return -107i32,
        };

        let  s = string.clone().to_vec();
        //println!("write state:{:?}",s);
        let res = match caller.data_mut().state.write_state(offset as u32, &s){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        res as i32
    } )?;
    //xq storage
    linker.func_wrap("concordium", "set_state",  |mut caller: Caller<'_, ContextStm<K>>, kstart: i32, klen: i32, vstart: i32, vlen: i32| ->i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        //println!("set_state param :K->{:?}, V->{:?}",kstart, vstart);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -106i32,
        };
        let  kdata = mem.data(&caller)
            .get(kstart as u32 as usize..)
            .and_then(|arr| arr.get(..klen as u32 as usize));
        let  kstring = match kdata {
            Some(s) => s,
            None => return -107i32,
        };

        let  k = kstring.clone().to_vec();

        let  vdata = mem.data(&caller)
            .get(vstart as u32 as usize..)
            .and_then(|arr| arr.get(..vlen as u32 as usize));
        let  vstring = match vdata {
            Some(s) => s,
            None => return -107i32,
        };

        let  v = vstring.clone().to_vec();
        //println!("set_stateb1:{:?}",k);
        //println!("set_stateb k:{:?},v:{:?}",kstring.clone(),vstring.clone());
        
        // let adr = caller.data().self_address.to_string();
        // let ad = match hex_str_to_bytes(&adr){
        //     Ok(mut a) => {a.truncate(20);a},
        //     Err(e) => {
        //         println!("wrset:hex_str_to_bytes!!!{:?}",e);
        //         return -108i32},
        // };
        let adr  = caller.data().self_address.0;
        let adra = &adr[2..22];
        //println!("{:?}",adr);
        //println!("{:?}",adra);
        let a:[u8;20] = adra.try_into().unwrap();
        let addr = AccountAddress::new(a);
        //println!("get_stateb2:{},gas:{:?} ",key.len(), &caller.data().gas_counter);
        let in_key = StateKey::AccessPath(AccessPath::new( addr,k.clone()));
        let op = WriteOp::Modification(v.clone());
        let _wr = caller.data_mut().writesets.insert((in_key,op));
        {
            // let key = match caller.data_mut().state.set_state(&k, &v){
            //     Ok(k) => k,
            //     Err(e) => return -108i32,
            // };
            
        }
        // let res = match caller.data_mut().state.set_state(&k, &v){
        //     Ok(res) => res,
        //     Err(e) => return -108i32,
        // };
        1 as i32
    } )?;
    //xq storage
    linker.func_wrap("concordium", "get_state", |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32, vstart: i32|-> i32 {
        //println!("gas0:{:?}",&caller.data().gas_counter);
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        //println!("gas:{:?}",&caller.data().gas_counter);

        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -106i32,
        };
        let  kdata = mem.data(&caller)
            .get(start as u32 as usize..)
            .and_then(|arr| arr.get(..length as u32 as usize));
        let  kstring = match kdata {
            Some(s) => s,
            None => return -107i32,
        };

        let  k = kstring.clone().to_vec();
        // let key:Vec<u8>= match caller.data_mut().state.get_state(&k){
        //     Ok(k) => k,
        //     Err(e) => return -108i32,
        // };
        
        //println!("get_stateb1:{:?}, gas:{:?}",key, &caller.data().gas_counter);
        // let mem = match caller.get_export("memory"){
        //     Some(Extern::Memory(mem)) => mem,
        //     _ => return 0,
        // };
        //let adr = caller.data().self_address.to_string();
        //println!("wasmstm:get_state ");
        // let ad = match hex_str_to_bytes(&adr){
        //     Ok(mut a) => {a.truncate(20);a},
        //     Err(e) => {
        //         println!("wasmstm:get_state error");
        //         return -108i32},
        // };
        let adr  = caller.data().self_address.0;
        let adra = &adr[2..22];
        //println!("{:?}",adr);
        //println!("{:?}",adra);
        let a:[u8;20] = adra.try_into().unwrap();
        let addr = AccountAddress::new(a);
        //println!("get_stateb2:{},gas:{:?} ",key.len(), &caller.data().gas_counter);
        let in_key = StateKey::AccessPath(AccessPath::new( addr,k.clone()));
        let cache = match caller.data().mvhashview.read(&in_key){
            block_executor::executor::ReadResult::Value(v) => {
                let c = match v.as_ref(){
                    WriteOp::Creation(value) => value.clone(),
                    WriteOp::Modification(value) => value.clone(),
                    WriteOp::Deletion => todo!(),
                };
                c
            },
            block_executor::executor::ReadResult::U128(_) => todo!(),
            block_executor::executor::ReadResult::Unresolved(_) => todo!(),
            block_executor::executor::ReadResult::None => {
                let key:Vec<u8>= match caller.data_mut().state.get_state(&k){
                    Ok(k) => k,
                    Err(e) => return -108i32,
                };
                key
            },
        };
        // let  _data = match mem.write(caller, vstart as usize, &key.as_slice() ){
        //     Ok(k) => k,
        //     Err(e) => return -108i32,
        // };

        let  _data = match mem.write(caller, vstart as usize, &cache.as_slice() ){
            Ok(k) => k,
            Err(e) => return -108i32,
        };

        cache.len() as i32
    })?;

    linker.func_wrap("concordium", "write_return",  |mut caller: Caller<'_, ContextStm<K>>, start: i32, len: i32, offset: i32| ->i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -104i32,
        };
        let  data = mem.data(&caller)
            .get(start as u32 as usize..)
            .and_then(|arr| arr.get(..len as u32 as usize));
        let  string = match data {
            Some(s) => s,
            None => return -105i32,
        };

        let  s = string.clone().to_vec();
        let res = match caller.data_mut().returndata.write_return(offset as u32, &s){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        res as i32
    } )?;

    linker.func_wrap("concordium", "get_parameter_size", |mut caller: Caller<'_, ContextStm<K>> |-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let len = caller.data_mut().param.len() as i32;
        len
    })?;

    linker.func_wrap("concordium", "get_parameter_section", |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32, offset: i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);

        let _write_end = start + length;
        let end = std::cmp::min(offset + length , caller.data_mut().param.len() as i32);
        let s:&[u8] = &caller.data_mut().param[offset as usize ..end as usize];

        let b = s.to_owned();

        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return -103i32,
        };
        let  _data = mem.write(caller, start as usize, &b);
        length
    })?;

    linker.func_wrap("concordium", "get_receive_sender", |mut caller: Caller<'_, ContextStm<K>>, start: i32| {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let  send : &mut [u8;ADDRESS_SIZE+1]= &mut [5; ADDRESS_SIZE+1];
        let _sender = caller.data_mut().sender.serial::<&mut [u8]>(&mut (&mut send[..]))
            .map_err(|_| 0);

        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return Err(Trap::new("failed to find get_receive_sender memory")),
        };
        let  _data = mem.write(caller, start as usize, &send[..]);
        Ok(())
    })?;

    linker.func_wrap("concordium", "get_receive_invoker", |mut caller: Caller<'_, ContextStm<K>>, start: i32| {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let invoker = caller.data_mut().invoker;

        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return Err(Trap::new("failed to find get_receive_invoker memory")),
        };
        let  data = mem.write(caller, start as usize, &invoker.0).unwrap();
        Ok(())
    })?;

    linker.func_wrap("concordium", "get_receive_owner", |mut caller: Caller<'_, ContextStm<K>>, start: i32| {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let owner = caller.data_mut().owner;

        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return Err(Trap::new("failed to find get_receive_owner memory")),
        };
        let  data = match mem.write(caller, start as usize, &owner.0){
            Ok(res) => res,
            Err(e) => return Err(Trap::new("failed to write get_receive_owner memory")),
        };
        Ok(())
    })?;

    linker.func_wrap("concordium", "get_receive_self_address", |mut caller: Caller<'_, ContextStm<K>>, start: i32| {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let contract_addr = caller.data_mut().self_address;

        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return Err(Trap::new("failed to find get_receive_self_address memory")),
        };
        let  data = match mem.write(caller, start as usize, &contract_addr.0){
            Ok(res) => res,
            Err(e) => return Err(Trap::new("failed to write get_receive_self_address memory")),
        };
        Ok(())
    })?;

    linker.func_wrap("concordium", "get_receive_self_balance", |mut caller: Caller<'_, ContextStm<K>>|-> i64 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let t = caller.data_mut().self_balance.micro_gtu;
        //println!("get_receive_self_balance:{:?}", t);
        t as i64
    })?;

    linker.func_wrap("concordium", "accept", |mut caller: Caller<'_, ContextStm<K>>|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let o = caller.data_mut().outcomes.accept();
        o as i32
    })?;

    linker.func_wrap("concordium", "combine_and", |mut caller: Caller<'_, ContextStm<K>>, left:i32, right:i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let o = match caller.data_mut().outcomes.combine_and(left as u32, right as u32){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        o as i32
    })?;

    linker.func_wrap("concordium", "combine_or", |mut caller: Caller<'_, ContextStm<K>>, left:i32, right:i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let o = match caller.data_mut().outcomes.combine_or(left as u32, right as u32){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        o as i32
    })?;
    //transfer & contract :xq
    linker.func_wrap("concordium", "send", |mut caller: Caller<'_, ContextStm<K>>, addr_index:i32, rcv_start:i32, rcv_len:i32, amount:i64, par_start:i32, par_len:i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -1i32,
        };
        //transfer & contract :xq
        let  addr = mem.data(&caller)
            .get(addr_index as u32 as usize..)
            .and_then(|arr| arr.get(..ADDRESS_SIZE as u32 as usize));
        let  string = match addr {
            Some(s) => s,
            None => return 0,
        };
        let  addr = string.clone().to_vec();

        let  data = mem.data(&caller)
            .get(rcv_start as u32 as usize..)
            .and_then(|arr| arr.get(..rcv_len as u32 as usize));
        let  name = match data {
            Some(s) => s,
            None => return -1i32,
        };

        let rcv_name = name.clone().to_vec();
        let  data = mem.data(&caller)
            .get(par_start as u32 as usize..)
            .and_then(|arr| arr.get(..par_len as u32 as usize));
        let  param = match data {
            Some(s) => s,
            None => return -1i32,
        };

        let parameter = param.clone().to_vec();
        //transfer & contract :xq
        let ret = match caller.data_mut().outcomes.send(
            &addr,
            &rcv_name,
            amount as u64,
            &parameter,
        ){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        ret as i32
    })?;

    linker.func_wrap("concordium", "simple_transfer", |mut caller: Caller<'_, ContextStm<K>>,start:i32, amount:i64|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return 3i32,
        };

        let  data = mem.data(&caller)
            .get(start as u32 as usize..)
            .and_then(|arr| arr.get(..ADDRESS_SIZE as u32 as usize));
        let  string = match data {
            Some(s) => s,
            None => return 0,
        };
        let  addr = string.clone().to_vec();
        //println!("simple_transfer:{:?}@{:?}",addr,amount);
        let ret = match caller.data_mut().outcomes.simple_transfer( &addr,amount as u64){
            Ok(res) => res,
            Err(e) => return -108i32,
        };
        ret as i32
    })?;

    linker.func_wrap("concordium", "state_size", |mut caller: Caller<'_, ContextStm<K>> |-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let len = caller.data_mut().state.len() as i32;
        //println!("state_size:{}",len);
        len
    })?;

    linker.func_wrap("concordium", "resize_state", |mut caller: Caller<'_, ContextStm<K>>, size:i32 |-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let new_size = size as u32;
        let _old_size = caller.data_mut().state.len();

        let ret = caller.data_mut().state.resize_state(new_size);
        ret as i32
    })?;

    linker.func_wrap("concordium", "get_slot_time", |mut caller: Caller<'_, ContextStm<K>> |-> i64 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let t = caller.data_mut().metadata.slot_time.timestamp_millis();
        //println!("get_slot_time:{:?}", t);
        t as i64
    })?;

    linker.func_wrap("concordium", "get_block_height", |mut caller: Caller<'_, ContextStm<K>> |-> i64 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let t = caller.data_mut().metadata.height;
        println!("get_block_height:{:?}", t);
        t as i64
    })?;
    //xq tx hash
    linker.func_wrap("concordium", "tx_hash", |mut caller: Caller<'_, ContextStm<K>>, addr_index:i32 |-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let tx:String = caller.data_mut().metadata.tx_hash.clone();
        println!("get_tx:{:?}, ",tx);
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => return -106i32,
        };
        let  _data = mem.write(caller, addr_index as usize, &tx.as_bytes()).unwrap();
        println!("tx_hash:{:?}", tx.as_bytes().len());
        tx.as_bytes().len() as i32
    })?;

    linker.func_wrap("concordium", "load_state", |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32, offset: i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let buffer:&mut [u8;65536] = &mut [0;65536];
        let b = caller.data_mut().state.load_state(offset as u32, &mut buffer[..length as usize]).unwrap();
        //println!("load_stateb:{}, ",b);
        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return 0,
        };
        let  _data = mem.write(caller, start as usize, &buffer[..length as usize] );
        b as i32
    })?;

    linker.func_wrap("concordium", "get_policy_section", |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32, _offset: i32|-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        let policies = caller.data_mut().sender_policies.policies_to_bytes();

        let len = std::cmp::min(length, policies.len() as i32);
        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return 0,
        };
        let  _data = mem.write(caller, start as usize, &policies[..len as usize] );
        len as i32
    })?;

    linker.func_wrap("concordium", "log_event", |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32 |-> i32 {
        charge_gas(&mut caller, GAS_ENV_FUNC_BASE);
        if length <= constants::MAX_LOG_SIZE as i32 {
            // only charge if we actually log something.

            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -100i32,
            };
            let  data = mem.data(&caller)
                .get(start as u32 as usize..)
                .and_then(|arr| arr.get(..length as u32 as usize));
            let  string = match data {
                Some(s) => s,
                None => return -101i32,
            };

            let  s = string.clone().to_vec();
            let res = caller.data_mut().logs.log_event(s);
            res
        } else {
            // otherwise the cost is adequately reflected by just the cost of a function
            // call.
            -102i32
        }
    })?;

    linker.func_wrap("gas", "gas", |mut caller: Caller<'_, ContextStm<K>>, amount: u32| {

        if caller.data_mut().gas_outof{
            return ()
        }
        let prev = caller.data_mut().gas_counter;
        // if (prev <10000) || (prev > 800000){
        //     println!("$$$$$$$$$$$$$$$gas:{:?}",prev);
        // }

        let ret = match prev.checked_add(amount as u64) {
            // gas charge overflow protection
            None => false,
            Some(val) if val > caller.data_mut().gas_limit => false,
            Some(_) => {
                caller.data_mut().gas_counter = prev + amount as u64;
                true
            }
        };

        return if ret {
            ()
        } else {
            caller.data_mut().gas_outof = true;
            ()
        }
    })?;

    linker.func_wrap("concordium", "print", |mut caller: Caller<'_, ContextStm<K>>, ptr: i32, len: i32| {
        let mem = match caller.get_export("memory"){
            Some(Extern::Memory(mem)) => mem,
            _ => return println!("print get mem err"),
        };

        let  data = mem.data(&caller)
            .get(ptr as u32 as usize..)
            .and_then(|arr| arr.get(..len as u32 as usize));
        let  string = match data {
            Some(s) => s,
            None => return (),
        };
        println!("print:{:?} len:{:?}", string,len);

    })?;

    linker
        .func_wrap(
            "concordium",
            "validate_vc",
            |mut caller: Caller<'_, ContextStm<K>>, start: i32, length: i32| -> i32 {
                charge_gas(&mut caller, GAS_ENV_FUNC_BASE);

                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return -106i32,
                };

                let vc_u8 = match mem
                    .data(&caller)
                    .get(start as u32 as usize..)
                    .and_then(|arr| arr.get(..length as u32 as usize))
                {
                    Some(s) => s,
                    _ => return -107i32,
                };
                let vc_string = match std::str::from_utf8(vc_u8){
                    Ok(v) => v.to_string(),
                    Err(_) => return -107i32,
                };

                let ret: i32 =  caller.data_mut().did.validate_vc(vc_string);

                ret
            },
        )?;

    Ok(linker)
}

fn charge_gas<K:ModulePath>(caller: &mut Caller<'_, ContextStm<K>>, amount: u64){
    if !caller.data().gas{
        return ()
    }
    if caller.data_mut().gas_outof{
        return ()
    }
    let prev = caller.data_mut().gas_counter;
    let ret = match prev.checked_add(amount) {
        // gas charge overflow protection
        None => false,
        Some(val) if val > caller.data_mut().gas_limit => false,
        Some(_) => {
            caller.data_mut().gas_counter = prev + amount;
            true
        }
    };

    return if ret {
        ()
    } else {
        caller.data_mut().gas_outof = true;
        ()
    }
}

// aot
pub fn execute_transaction2<K:ModulePath>(func_name: &str, context: ContextStm<K>, binary: &[u8], amount: i64) -> Result<ReceiveResult> {

    let engine = Engine::default();
    let module = unsafe{Module::deserialize(&engine, binary)?};
    let linker = host_func_init(&engine)?;
    let mut store = Store::new(&engine, context);
    let instance = linker.instantiate(&mut store, &module)?;

    let run = match instance.get_func(&mut store, func_name) {
        Some(r) => r,
        None => return Err(anyhow!("can't find func from module!")),
    };

    let ret = match run.call(&mut store, &[I64(amount)]){
        Ok(r) => {
            Some(r[0].clone())
        },
        Err(trap) => {
            return Err(trap)
        },
    };

    if let Some(Val::I32(n)) = ret {
        if store.data().gas_outof{
            return Ok(ReceiveResult::OutOfEnergy)
        };
        let mut energy = store.data().gas_limit-store.data().gas_counter;
        energy = energy/GAS_SCALE;
        let mut actions = store.data().outcomes.cur_state.clone();
        if n >= 0 && (n as usize) < actions.len() {
            let n = n as usize;
            actions.truncate(n + 1);
            Ok(ReceiveResult::Success {
                logs: store.data().logs.clone(),
                state: store.data().state.clone(),
                actions,
                returndata: store.data().returndata.clone(),
                remaining_energy:energy,
            })
        } else if n >= 0 {
            bail!("Invalid return.")
        } else {
            Ok(ReceiveResult::Reject {
                reason: reason_from_wasm_error_code(n)?,
                remaining_energy:energy,
            })
        }
    } else {
        bail!(
            "Invalid return. Expected a value, but receive nothing. This should not happen for \
             well-formed modules"
        );
    }
}

pub fn execute_transaction<K:ModulePath>(func_name: &str, context: ContextStm<K>, binary: &[u8], amount: i64) -> Result<WasmTransactionOutput> {
    let kind = context.kind.clone();
    let engine = Engine::default();
    let module = unsafe{Module::deserialize(&engine, binary)?};
    let linker = host_func_init(&engine)?;
    let mut store = Store::new(&engine, context);
    let instance = linker.instantiate(&mut store, &module)?;

    let run = match instance.get_func(&mut store, func_name) {
        Some(r) => r,
        None => return Err(anyhow!("can't find func from module!")),
    };

    let ret = match run.call(&mut store, &[I64(amount)]){
        Ok(r) => {
            Some(r[0].clone())
        },
        Err(trap) => {
            return Err(trap)
        },
    };

    if let Some(Val::I32(n)) = ret {
        if store.data().gas_outof{
            return Ok(WasmTransactionOutput::Reject { reason: -1, remaining_gas: 0 })
        };
        let mut energy = store.data().gas_limit-store.data().gas_counter;
        energy = energy/GAS_SCALE;
        let mut actions = store.data().outcomes.cur_state.clone();
        //println!("n:{},action:{}",n,actions.len());
        if kind  == ExecKind::Init{
            if n == 0 {
                println!("state init:{:?}",store.data().state.clone());
                Ok(WasmTransactionOutput::Success {
                    logs: store.data().logs.clone(),
                    state: store.data().state.clone(),
                    actions,
                    returndata: store.data().returndata.clone(),
                    remaining_gas:energy,
                    write_set: store.data().writesets.clone(),
                })
            }  else {
                Ok(WasmTransactionOutput::Reject {
                    reason: reason_from_wasm_error_code(n)?,
                    remaining_gas:energy,
                })
            }
        }else{
            if n >= 0 && (n as usize) < actions.len() {
                let n = n as usize;
                actions.truncate(n + 1);
                //println!("state call:{:?}",store.data().state.clone());
                Ok(WasmTransactionOutput::Success {
                    logs: store.data().logs.clone(),
                    state: store.data().state.clone(),
                    actions,
                    returndata: store.data().returndata.clone(),
                    remaining_gas:energy,
                    write_set: store.data().writesets.clone(),
                })
            } else if n >= 0 {
                println!("invalid return:{}",n);
                bail!("Invalid return.")
            } else {
                Ok(WasmTransactionOutput::Reject {
                    reason: reason_from_wasm_error_code(n)?,
                    remaining_gas:energy,
                })
            }
        }
    } else {
        bail!(
            "Invalid return. Expected a value, but receive nothing. This should not happen for \
             well-formed modules"
        );
    }
}


