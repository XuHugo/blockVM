use wasm_chain_integration::*;
use concordium_contracts_common::{ Amount, Address, ChainMetadata, Timestamp, AccountAddress, DID};
use anyhow::{Error};
use std::{
    fs,
    sync::Arc,
};

/// Size of an account address when serialized in binary.
/// NB: This is different from the Base58 representation.
pub const ADDRESS_SIZE: usize = 42;
pub const GAS_SCALE: u64 = 10;
pub const GAS_ENV_FUNC_BASE: u64 = 20*GAS_SCALE;
pub const GAS_INIT_FUNC_BASE: u64 = 200*GAS_SCALE;

#[derive(Debug)]
pub enum ContractResult {
    Success {
        remaining_energy: u64,
        actions:          Vec<Action>,
        event:            Logs,
    },
    Data {
        data: ReturnData,
        remaining_energy: u64,
        event:            Logs,
    },
    Reject {
        reason:           i32,
        remaining_energy: u64,
    },
    OutOfEnergy,
}

#[derive(Debug, derive_more::Display)]
pub enum ContractError {
    /// Code could not be read from the state.
    CodeNotFound,
    /// Wasm code failed validation.
    InvalidModule,
    /// Wasm code could not be deserialized.
    CantDeserializeWasm,
    /// The module does not export a linear memory named `memory`.
    InvalidMemory,
    /// The number of heap pages requested is disallowed by the module.
    InvalidHeapPages,
    /// Instantiation error.
    Instantiation(String),
    /// Other error happenend.
    Other(String),
}

impl std::error::Error for ContractError {}

// fn bool2i32(t: bool) -> Result<i32, Error>{
//     return if t {
//         Ok(1)
//     }else{
//         Ok(0)
//     }
// }
//
// #[derive(Clone)]
// pub struct DID{}
//
// impl Copy for DID{}
// impl DID {
//     pub fn validate_vc(self, vc:string) -> i32{
//
//         return 1i32
//     }
// }

#[derive(Clone,PartialEq)]
pub enum ExecKind {
    Init,
    Call,
}

#[derive(Clone)]
pub enum ContractKind {
    Concordium,
    Solidity,
    Substrate,
}

#[derive(Clone)]
pub enum VMKind {
    WasmTime,
    GeeCo,
    WasmTimeAOT,
    //JVM,
}


#[derive(Clone)]
pub enum FunName{
    InitName(String),
    CallName(String, String),
}

#[derive(Clone)]
pub enum VMType{
    WasmTime,
    GeeCo,
    JVM,
}

#[derive(Clone)]
pub struct Context<'a> {
    pub(crate) kind: ExecKind,
    pub(crate) func_name: FunName,
    pub(crate) origin: [u8; ADDRESS_SIZE],
    pub(crate) state: State,
    pub(crate) param: &'a [u8],
    pub(crate) outcomes: Outcome,
    pub(crate) sender:  Address,
    pub(crate) invoker:  AccountAddress,
    pub(crate) owner:  AccountAddress,
    pub(crate) logs: Logs,
    pub(crate) self_balance: Amount,
    pub(crate) self_address: AccountAddress,
    pub(crate) metadata: ChainMetadata,
    pub(crate) sender_policies: &'a [u8],
    pub(crate) returndata: ReturnData,
    pub(crate) gas_counter: u64,
    pub(crate) gas_limit: u64,
    pub(crate) gas_outof: bool,
    //pub(crate) store: Store,
    pub gas: bool,
    pub(crate) did: DID,
}

impl<'a> Context <'a>{
    pub fn new() -> Self {
        Context{
            kind: ExecKind::Init,
            func_name: FunName::InitName("contract_name".to_string()),
            origin: [0u8; ADDRESS_SIZE],
            state: State::new(None),
            param:&[],
            outcomes: Outcome::new(),
            sender: Address::Account(concordium_contracts_common::AccountAddress([0u8; ADDRESS_SIZE])),
            invoker:  concordium_contracts_common::AccountAddress([0u8; ADDRESS_SIZE]),
            owner:  concordium_contracts_common::AccountAddress([0u8; ADDRESS_SIZE]),
            logs: Logs::new(),
            self_balance: Amount::zero(),
            self_address: concordium_contracts_common::AccountAddress([0u8; ADDRESS_SIZE]),
            metadata:        ChainMetadata {
                slot_time: Timestamp::from_timestamp_millis(0),
                height: 99998,
                tx_hash: String::from("80b0c76da4ad03e5559773ed89783ca778fe71525c67cf4002ce1f3406e0c5e0"),

            },
            sender_policies: &[],
            returndata: ReturnData::new(None),
            gas_counter: 0,
            gas_limit:0,
            gas_outof: false,
            gas:false,
            did:DID{},
        }
    }
    pub fn new_init(contract_name: &str, origin:AccountAddress, param: &'a [u8], balance:Amount, contract_addr:AccountAddress,gas_limit:u64, gas:bool) -> Self{
        Context{
            kind: ExecKind::Init,
            func_name: FunName::InitName(contract_name.to_string()),
            origin: origin.0,
            state: State::new(None),
            param,
            outcomes: Outcome::new(),
            sender: Address::Account(concordium_contracts_common::AccountAddress([0u8; ADDRESS_SIZE])),
            invoker:  origin,
            owner:  origin,
            logs: Logs::new(),
            self_balance: balance,
            self_address: contract_addr,
            metadata:        ChainMetadata {
                slot_time: Timestamp::from_timestamp_millis(0),
                height: 99998,
                tx_hash: String::from("80b0c76da4ad03e5559773ed89783ca778fe71525c67cf4002ce1f3406e0c5e0"),

            },
            sender_policies: &[],
            returndata: ReturnData::new(None),
            gas_counter: 0,
            gas_limit:gas_limit*GAS_SCALE,
            gas_outof: false,
            gas,
            did:DID{},
        }
    }

    pub fn new_call(contract_name: &str,
                func_name: &str,
                param: &'a [u8],
                state: &'a [u8],
                balance:Amount,
                sender: Address,
                invoker:AccountAddress,
                owner:AccountAddress,
                contract_addr:AccountAddress,
                    gas_limit:u64,
                gas: bool,
    ) -> Self{
        Context{
            kind: ExecKind::Call,
            func_name: FunName::CallName(contract_name.to_string(), func_name.to_string()),
            origin: [0u8;ADDRESS_SIZE],
            state: State::new(Some(state)),
            param,
            outcomes: Outcome::new(),
            sender,
            invoker,
            owner,
            logs: Logs::new(),
            self_balance: balance,
            self_address: contract_addr,
            metadata:       ChainMetadata {
                slot_time: Timestamp::from_timestamp_millis(0),
                height: 99998,
                tx_hash: String::from("80b0c76da4ad03e5559773ed89783ca778fe71525c67cf4002ce1f3406e0c5e0"),
            },
            sender_policies: &[],
            returndata:ReturnData::new(None),
            gas_counter: 0,
            gas_limit:gas_limit*GAS_SCALE,
            gas_outof: false,
            gas,
            did:DID{},
        }
    }
}

#[derive(Clone)]
pub struct Context_JVM {
    pub(crate) version:u64,
    pub(crate) kind: ExecKind,
    pub(crate) contract_name: String,
    pub(crate) func_name: String,
    pub(crate) param: String,
    pub(crate) sender:  String,
    pub(crate) invoker:  String,
    pub(crate) owner:  String,
    pub(crate) logs: String,
    pub(crate) self_balance: u64,
    pub(crate) self_address: String,
    pub(crate) metadata: ChainMetadata,
    pub(crate) returndata: String,
    pub(crate) gas_counter: u64,
    pub(crate) gas_limit: u64,
    pub(crate) gas_outof: bool,
    //pub(crate) store: Store,
    pub(crate) gas: bool,
    pub(crate) did: DID,
}

impl Context_JVM{
    pub fn new(
        version:u64,
        kind:ExecKind,
        contract_name: String,
        func_name: String,
        param: String,
        balance:u64,
        sender: String,
        invoker:String,
        owner:String,
        contract_addr:String,
        gas_limit:u64,
        gas: bool,
    ) -> Self{
        Context_JVM{
            version,
            kind,
            contract_name,
            func_name,
            param,
            sender,
            invoker,
            owner,
            logs: String::new(),
            self_balance: balance,
            self_address: contract_addr,
            metadata:       ChainMetadata {
                slot_time: Timestamp::from_timestamp_millis(0),
                height: 999985,
                tx_hash: String::from("80b0c76da4ad03e5559773ed89783ca778fe71525c67cf4002ce1f3406e0c5e0"),
            },
            returndata:String::new(),
            gas_counter: 0,
            gas_limit:gas_limit*GAS_SCALE,
            gas_outof: false,
            gas,
            did:DID{},
        }
    }
}

#[derive(Clone, Debug)]
pub struct Result_JVM {
    pub logs: String,
    pub returndata: String,
    pub gas_counter: u64,
    pub gas_outof: bool,
}

use block_executor::{
    txn_last_input_output::ReadDescriptor,
    task::ModulePath,
    executor::MVHashMapView,
    scheduler::Scheduler,
    mutex::Mutex,
};
use aptos_types::{
    write_set::WriteSet, 
    state_store::state_key::StateKey, 
    write_set::WriteOp
};

use  geeco_mvhashmap::MVHashMap;

//#[derive(Clone)]
pub struct ContextStm<'a, K> {
    pub(crate) kind: ExecKind,
    pub(crate) func_name: FunName,
    pub(crate) origin: [u8; ADDRESS_SIZE],
    pub(crate) state: State,
    pub(crate) param: &'a [u8],
    pub(crate) outcomes: Outcome,
    pub(crate) sender:  Address,
    pub(crate) invoker:  AccountAddress,
    pub(crate) owner:  AccountAddress,
    pub(crate) logs: Logs,
    pub(crate) self_balance: Amount,
    pub(crate) self_address: AccountAddress,
    pub(crate) metadata: ChainMetadata,
    pub(crate) sender_policies: &'a [u8],
    pub(crate) returndata: ReturnData,
    pub(crate) gas_counter: u64,
    pub(crate) gas_limit: u64,
    pub(crate) gas_outof: bool,
    pub gas: bool,
    pub(crate) did: DID,
    pub(crate) readsets: Vec<ReadDescriptor<K>>,
    pub(crate) writesets: WriteSet,
    pub(crate) mvhashview:&'a MVHashMapView<'a, StateKey, WriteOp>,
}


impl<'a, K:ModulePath> ContextStm <'a, K>{
pub fn new_call(kind: ExecKind,
    contract_name: &str,
    func_name: &str,
    param: &'a [u8],
    balance:Amount,
    invoker:AccountAddress,
    owner:AccountAddress,
    contract_addr:AccountAddress,
    gas_limit:u64,
    gas: bool,
    mvhashview:&'a MVHashMapView<StateKey, WriteOp>,
    state:&'a[u8]
) -> Self{
    ContextStm{
        kind,
        func_name: FunName::CallName(contract_name.to_string(), func_name.to_string()),
        origin: [0u8;ADDRESS_SIZE],
        state: State::new(Some(state)),
        param,
        outcomes: Outcome::new(),
        sender:Address::Account(invoker),
        invoker,
        owner,
        logs: Logs::new(),
        self_balance: balance,
        self_address: contract_addr,
        metadata:       ChainMetadata {
            slot_time: Timestamp::from_timestamp_millis(0),
            height: 99998,
            tx_hash: String::from("80b0c76da4ad03e5559773ed89783ca778fe71525c67cf4002ce1f3406e0c5e0"),
        },
        sender_policies: &[],
        returndata:ReturnData::new(None),
        gas_counter: 0,
        gas_limit:gas_limit*GAS_SCALE,
        gas_outof: false,
        gas,
        did:DID{},
        readsets: vec![],
        writesets: Default::default(),
        mvhashview,
    }
}
}


// pub struct GeecoTransactionOutput {
//     data: ReturnData,
//     write_set: WriteSet,
//     events: Vec<ContractEvent>,
//     gas_used: u64,
//     status: TransactionStatus,
// }


#[derive(Debug)]
pub enum WasmTransactionOutput {
    Success {
        write_set:        WriteSet,
        state:            State,
        logs:             Logs,
        actions:          Vec<Action>,
        returndata:       ReturnData,
        remaining_gas: u64,
    },
    Reject {
        reason:           i32,
        remaining_gas: u64,
    },
}
