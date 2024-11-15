pub mod constants;
#[cfg(feature = "enable-ffi")]
mod ffi;

#[cfg(test)]
mod validation_tests;

use anyhow::{anyhow, bail, ensure};
use concordium_contracts_common::*;
use constants::*;
use machine::Value;
use std::{
    collections::{BTreeMap, LinkedList},
    convert::TryInto,
    io::Write,
};
pub use types::*;
use wasm_transform::{
    artifact::{Artifact, ArtifactNamedImport, RunnableCode, TryFromImport},
    machine,
    parse::{parse_custom, parse_skeleton},
    types::{ExportDescription, Module, Name},
    utils, validate,
};
//xq storage test
use storage::{StorageInstanceRef};

#[cfg(feature = "fuzz")]
pub mod fuzz;
mod types;

pub type ExecResult<A> = anyhow::Result<A>;

#[derive(Clone, Debug, Default)]
/// Structure to support logging of events from smart contracts.
pub struct Logs {
    pub logs: LinkedList<Vec<u8>>,
}

impl Logs {
    pub fn new() -> Self {
        Self {
            logs: LinkedList::new(),
        }
    }

    /// The return value is
    ///
    /// - 0 if data was not logged because it would exceed maximum number of
    ///   logs
    /// - 1 if data was logged.
    pub fn log_event(&mut self, event: Vec<u8>) -> i32 {
        let cur_len = self.logs.len();
        if cur_len < constants::MAX_NUM_LOGS {
            self.logs.push_back(event);
            1
        } else {
            0
        }
    }

    pub fn iterate(&self) -> impl Iterator<Item = &Vec<u8>> { self.logs.iter() }

    pub fn to_bytes(&self) -> Vec<u8> {
        let len = self.logs.len();
        let mut out = Vec::with_capacity(4 * len + 4);
        out.extend_from_slice(&(len as u32).to_be_bytes());
        for v in self.iterate() {
            out.extend_from_slice(&(v.len() as u32).to_be_bytes());
            out.extend_from_slice(v);
        }
        out
    }

    pub fn log_num(&mut self) -> i32 {
        let cur_len = self.logs.len();
        cur_len as i32
    }
}

#[derive(Clone, Copy)]
pub struct Energy {
    /// Energy left to use
    pub energy: u64,
}

/// Cost of allocation of one page of memory in relation to execution cost.
/// FIXME: It is unclear whether this is really necessary with the hard limit we
/// have on memory use.
/// If we keep it, the cost must be analyzed and put into perspective
pub const MEMORY_COST_FACTOR: u32 = 100;

#[derive(Debug)]
pub struct OutOfEnergy;

impl std::fmt::Display for OutOfEnergy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { "Out of energy".fmt(f) }
}

impl Energy {
    pub fn tick_energy(&mut self, amount: u64) -> ExecResult<()> {
        if self.energy >= amount {
            self.energy -= amount;
            Ok(())
        } else {
            self.energy = 0;
            bail!(OutOfEnergy)
        }
    }

    /// TODO: This needs more specification. At the moment it is not used, but
    /// should be.
    pub fn charge_stack(&mut self, amount: u64) -> ExecResult<()> {
        if self.energy >= amount {
            self.energy -= amount;
            Ok(())
        } else {
            self.energy = 0;
            bail!("Out of energy.")
        }
    }

    /// Charge energy for allocating the given number of pages.
    /// Since there is a hard limit on the amount of memory this is not so
    /// essential. The base cost of calling this host function is already
    /// covered by the metering transformation, hence if num_pages=0 it is
    /// OK for this function to charge nothing.
    ///
    /// This function will charge regardless of whether memory allocation
    /// actually happens, i.e., even if growing the memory would go over the
    /// maximum. This is OK since trying to allocate too much memory is likely
    /// going to lead to program failure anyhow.
    pub fn charge_memory_alloc(&mut self, num_pages: u32) -> ExecResult<()> {
        let to_charge = u64::from(num_pages) * u64::from(MEMORY_COST_FACTOR); // this cannot overflow because of the cast.
        self.tick_energy(to_charge)
    }
}

#[derive(Clone, Default)]
/// The Default instance of this type constructs and empty list of outcomes.
pub struct Outcome {
    pub cur_state: Vec<Action>,
}

impl Outcome {
    pub fn new() -> Outcome { Self::default() }

    pub fn accept(&mut self) -> u32 {
        let response = self.cur_state.len();
        self.cur_state.push(Action::Accept);
        response as u32
    }

    pub fn get(&mut self) -> u32 {
        let response = self.cur_state.len();
        self.cur_state.push(Action::Get);
        response as u32
    }

    pub fn simple_transfer(&mut self, bytes: &[u8], amount: u64) -> ExecResult<u32> {
        let response = self.cur_state.len();
        let addr: [u8; ACCOUNT_ADDRESS_SIZE] = bytes.try_into()?;
        let to_addr = AccountAddress(addr);
        let data = std::rc::Rc::new(SimpleTransferAction {
            to_addr,
            amount,
        });
        self.cur_state.push(Action::SimpleTransfer {
            data,
        });
        Ok(response as u32)
    }

    //transfer & contract :xq
    pub fn send(
        &mut self,
        addr_index:  &[u8],
        receive_name_bytes: &[u8],
        amount: u64,
        parameter_bytes: &[u8],
    ) -> ExecResult<u32> {
        let response = self.cur_state.len();

        let name_str = std::str::from_utf8(receive_name_bytes)?;
        ensure!(ReceiveName::is_valid_receive_name(name_str).is_ok(), "Not a valid receive name.");
        let name = receive_name_bytes.to_vec();

        ensure!(parameter_bytes.len() <= MAX_PARAMETER_SIZE, "Parameter exceeds max size.");

        let parameter = parameter_bytes.to_vec();
        //transfer & contract :xq
        let addr: [u8; ACCOUNT_ADDRESS_SIZE] = addr_index.try_into()?;
        let to_addr = AccountAddress(addr);

        let data = std::rc::Rc::new(SendAction {
            to_addr,
            name,
            amount,
            parameter,
        });
        self.cur_state.push(Action::Send {
            data,
        });
        Ok(response as u32)
    }

    pub fn combine_and(&mut self, l: u32, r: u32) -> ExecResult<u32> {
        let response = self.cur_state.len() as u32;
        ensure!(l < response && r < response, "Combining unknown actions.");
        self.cur_state.push(Action::And {
            l,
            r,
        });
        Ok(response)
    }

    pub fn combine_or(&mut self, l: u32, r: u32) -> ExecResult<u32> {
        let response = self.cur_state.len() as u32;
        ensure!(l < response && r < response, "Combining unknown actions.");
        self.cur_state.push(Action::Or {
            l,
            r,
        });
        Ok(response)
    }
}
// //xq storage
// #[derive(Clone, Debug)]
// pub struct  Store{
//     pub af: &mut AccountFrame,
// }
//
// impl Store{
//     //xq storage
//     pub fn set_state(&mut self, kbytes: &[u8], vbytes: &[u8]) -> ExecResult<u32> {
//         let db = StorageInstanceRef.write().account_db();
//         println!("DB:set_state_{:?}:{:?}",kbytes,vbytes);
//         db.lock().put_bytes(&kbytes, &vbytes.to_vec());
//
//         Ok(vbytes.len() as u32)
//     }
//     //xq storage
//     pub fn get_state(&mut self, kbytes: &[u8]) -> ExecResult<Vec<u8>> {
//         let db = StorageInstanceRef.write().account_db();
//         let mut d:Vec<u8> = Vec::new();
//         db.lock().get_bytes(&kbytes, &mut d);
//         println!("DB:get_state_{:?}:{:?}",kbytes,d);
//         Ok(d)
//     }
// }

/// Smart contract state.
#[derive(Clone, Debug)]
pub struct State {
    pub state: Vec<u8>,
}

impl State {
    pub fn is_empty(&self) -> bool { self.state.is_empty() }

    // FIXME: This should not be copying so much data around, but for POC it is
    // fine. We should probably do some sort of copy-on-write here in the near term,
    // and in the long term we need to keep track of which parts were written.
    pub fn new(st: Option<&[u8]>) -> Self {
        match st {
            None => Self {
                state: Vec::new(),
            },
            Some(bytes) => Self {
                state: Vec::from(bytes),
            },
        }
    }

    pub fn len(&self) -> u32 { self.state.len() as u32 }

    pub fn write_state(&mut self, offset: u32, bytes: &[u8]) -> ExecResult<u32> {
        let length = bytes.len();
        ensure!(offset <= self.len(), "Cannot write past the offset.");
        let offset = offset as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| anyhow!("Writing past the end of memory."))? as usize;
        let end = std::cmp::min(end, MAX_CONTRACT_STATE as usize) as u32;
        if self.len() < end {
            self.state.resize(end as usize, 0u8);
        }
        let written = (&mut self.state[offset..end as usize]).write(bytes)?;
        Ok(written as u32)
    }

    pub fn load_state(&self, offset: u32, mut bytes: &mut [u8]) -> ExecResult<u32> {
        let offset = offset as usize;
        ensure!(offset <= self.state.len());
        // Write on slices overwrites the buffer and returns how many bytes were
        // written.
        let amt = bytes.write(&self.state[offset..])?;
        Ok(amt as u32)
    }

    pub fn resize_state(&mut self, new_size: u32) -> u32 {
        if new_size > MAX_CONTRACT_STATE {
            0
        } else {
            self.state.resize(new_size as usize, 0u8);
            1
        }
    }
    //xq storage
    pub fn set_state(&mut self, kbytes: &[u8], vbytes: &[u8]) -> ExecResult<u32> {
        let db = StorageInstanceRef.write().account_db();
        //println!("DB:set_state_{:?}:{:?}",kbytes,vbytes);
        db.lock().put_bytes(&kbytes, &vbytes.to_vec());

        Ok(vbytes.len() as u32)
    }
    //xq storage
    pub fn get_state(&mut self, kbytes: &[u8]) -> ExecResult<Vec<u8>> {
        let db = StorageInstanceRef.write().account_db();
        let mut d:Vec<u8> = Vec::new();
        db.lock().get_bytes(&kbytes, &mut d);
        //println!("DB:get_state_{:?}:{:?}",kbytes,d);
        Ok(d)
    }
}

#[derive(Clone, Debug)]
pub struct ReturnData {
    pub returndata: Vec<u8>,
}

impl ReturnData {
    pub fn len(&self) -> u32 { self.returndata.len() as u32 }

    pub fn new(st: Option<&[u8]>) -> Self {
        match st {
            None => Self {
                returndata: Vec::new(),
            },
            Some(bytes) => Self {
                returndata: Vec::from(bytes),
            },
        }
    }

    pub fn write_return(&mut self, offset: u32, bytes: &[u8]) -> ExecResult<u32> {
        let length = bytes.len();
        ensure!(offset <= self.len(), "Cannot write past the offset.");
        let offset = offset as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| anyhow!("Writing past the end of memory."))? as usize;
        let end = std::cmp::min(end, MAX_CONTRACT_STATE as usize) as u32;
        //let written = (&mut self.returndata[offset..end as usize]).write(bytes)?;
        let written = &mut self.returndata.extend(bytes);
        Ok(length as u32)
    }


}
pub struct InitHost<'a, Ctx> {
    /// Remaining energy for execution.
    pub energy:            Energy,
    /// Remaining amount of activation frames.
    /// In other words, how many more functions can we call in a nested way.
    pub activation_frames: u32,
    /// Logs produced during execution.
    pub logs:              Logs,
    /// The contract's state.
    pub state:             State,
    /// The parameter to the init method.
    pub param:             &'a [u8],
    /// The init context for this invocation.
    pub init_ctx:          &'a Ctx,
    /// The result to the init method.
    pub result:             ReturnData,
}

pub struct ReceiveHost<'a, Ctx> {
    /// Remaining energy for execution.
    pub energy:            Energy,
    /// Remaining amount of activation frames.
    /// In other words, how many more functions can we call in a nested way.
    pub activation_frames: u32,
    /// Logs produced during execution.
    pub logs:              Logs,
    /// The contract's state.
    pub state:             State,
    /// The parameter to the init method.
    pub param:             &'a [u8],
    /// Outcomes of the execution, i.e., the actions tree.
    pub outcomes:          Outcome,
    /// The receive context for this call.
    pub receive_ctx:       &'a Ctx,
    /// The result to the receive method.
    pub result:             ReturnData,
}

pub trait HasCommon {
    type MetadataType: HasChainMetadata;
    type PolicyBytesType: AsRef<[u8]>;
    type PolicyType: SerialPolicies<Self::PolicyBytesType>;

    fn energy(&mut self) -> &mut Energy;
    fn logs(&mut self) -> &mut Logs;
    fn state(&mut self) -> &mut State;
    fn param(&self) -> &[u8];
    fn policies(&self) -> ExecResult<&Self::PolicyType>;
    fn metadata(&self) -> &Self::MetadataType;
    fn result(&mut self) -> &mut ReturnData;
    fn did(&mut self) -> DID;
}

impl<'a, Ctx: HasInitContext> HasCommon for InitHost<'a, Ctx> {
    type MetadataType = Ctx::MetadataType;
    type PolicyBytesType = Ctx::PolicyBytesType;
    type PolicyType = Ctx::PolicyType;

    fn energy(&mut self) -> &mut Energy { &mut self.energy }

    fn logs(&mut self) -> &mut Logs { &mut self.logs }

    fn state(&mut self) -> &mut State { &mut self.state }

    fn param(&self) -> &[u8] { &self.param }

    fn metadata(&self) -> &Self::MetadataType { self.init_ctx.metadata() }

    fn policies(&self) -> ExecResult<&Self::PolicyType> { self.init_ctx.sender_policies() }

    fn result(&mut self) -> &mut ReturnData { &mut self.result }

    fn did(&mut self) -> DID {DID{}}
}

impl<'a, Ctx: HasReceiveContext> HasCommon for ReceiveHost<'a, Ctx> {
    type MetadataType = Ctx::MetadataType;
    type PolicyBytesType = Ctx::PolicyBytesType;
    type PolicyType = Ctx::PolicyType;

    fn energy(&mut self) -> &mut Energy { &mut self.energy }

    fn logs(&mut self) -> &mut Logs { &mut self.logs }

    fn state(&mut self) -> &mut State { &mut self.state }

    fn param(&self) -> &[u8] { &self.param }

    fn metadata(&self) -> &Self::MetadataType { self.receive_ctx.metadata() }

    fn policies(&self) -> ExecResult<&Self::PolicyType> { self.receive_ctx.sender_policies() }

    fn result(&mut self) -> &mut ReturnData { &mut self.result }

    fn did(&mut self) -> DID {DID{}}
}

/// Types which can act as init contexts.
///
/// Used to enable partial JSON contexts when simulating contracts with
/// cargo-concordium.
///
/// We have two implementations:
///  - `InitContext`, which is used on-chain and always returns `Ok(..)`.
///  - `InitContextOpt`, which is used during simulation with cargo-concordium
///    and returns `Ok(..)` for fields supplied in a JSON context, and `Err(..)`
///    otherwise.
pub trait HasInitContext {
    type MetadataType: HasChainMetadata;
    type PolicyBytesType: AsRef<[u8]>;
    type PolicyType: SerialPolicies<Self::PolicyBytesType>;

    fn metadata(&self) -> &Self::MetadataType;
    fn init_origin(&self) -> ExecResult<&AccountAddress>;
    fn sender_policies(&self) -> ExecResult<&Self::PolicyType>;
}

impl HasInitContext for InitContext<Vec<OwnedPolicy>> {
    type MetadataType = ChainMetadata;
    type PolicyBytesType = Vec<u8>;
    type PolicyType = Vec<OwnedPolicy>;

    fn metadata(&self) -> &Self::MetadataType { &self.metadata }

    fn init_origin(&self) -> ExecResult<&AccountAddress> { Ok(&self.init_origin) }

    fn sender_policies(&self) -> ExecResult<&Self::PolicyType> { Ok(&self.sender_policies) }
}

impl<'a> HasInitContext for InitContext<&'a [u8]> {
    type MetadataType = ChainMetadata;
    type PolicyBytesType = &'a [u8];
    type PolicyType = &'a [u8];

    fn metadata(&self) -> &Self::MetadataType { &self.metadata }

    fn init_origin(&self) -> ExecResult<&AccountAddress> { Ok(&self.init_origin) }

    fn sender_policies(&self) -> ExecResult<&Self::PolicyType> { Ok(&self.sender_policies) }
}

/// Types which can act as receive contexts.
///
/// Used to enable partial JSON contexts when simulating contracts with
/// cargo-concordium.
///
/// We have two implementations:
///  - `ReceiveContext`, which is used on-chain and always returns `Ok(..)`.
///  - `ReceiveContextOpt`, which is used during simulation with
///    cargo-concordium and returns `Ok(..)` for fields supplied in a JSON
///    context, and `Err(..)` otherwise.
pub trait HasReceiveContext {
    type MetadataType: HasChainMetadata;
    type PolicyBytesType: AsRef<[u8]>;
    type PolicyType: SerialPolicies<Self::PolicyBytesType>;

    fn metadata(&self) -> &Self::MetadataType;
    fn invoker(&self) -> ExecResult<&AccountAddress>;
    fn self_address(&self) -> ExecResult<&AccountAddress>;
    fn self_balance(&self) -> ExecResult<Amount>;
    fn sender(&self) -> ExecResult<&Address>;
    fn owner(&self) -> ExecResult<&AccountAddress>;
    fn sender_policies(&self) -> ExecResult<&Self::PolicyType>;
}

impl HasReceiveContext for ReceiveContext<Vec<OwnedPolicy>> {
    type MetadataType = ChainMetadata;
    type PolicyBytesType = Vec<u8>;
    type PolicyType = Vec<OwnedPolicy>;

    fn metadata(&self) -> &Self::MetadataType { &self.metadata }

    fn invoker(&self) -> ExecResult<&AccountAddress> { Ok(&self.invoker) }

    fn self_address(&self) -> ExecResult<&AccountAddress> { Ok(&self.self_address) }

    fn self_balance(&self) -> ExecResult<Amount> { Ok(self.self_balance) }

    fn sender(&self) -> ExecResult<&Address> { Ok(&self.sender) }

    fn owner(&self) -> ExecResult<&AccountAddress> { Ok(&self.owner) }

    fn sender_policies(&self) -> ExecResult<&Self::PolicyType> { Ok(&self.sender_policies) }
}

impl<'a> HasReceiveContext for ReceiveContext<&'a [u8]> {
    type MetadataType = ChainMetadata;
    type PolicyBytesType = &'a [u8];
    type PolicyType = &'a [u8];

    fn metadata(&self) -> &Self::MetadataType { &self.metadata }

    fn invoker(&self) -> ExecResult<&AccountAddress> { Ok(&self.invoker) }

    fn self_address(&self) -> ExecResult<&AccountAddress> { Ok(&self.self_address) }

    fn self_balance(&self) -> ExecResult<Amount> { Ok(self.self_balance) }

    fn sender(&self) -> ExecResult<&Address> { Ok(&self.sender) }

    fn owner(&self) -> ExecResult<&AccountAddress> { Ok(&self.owner) }

    fn sender_policies(&self) -> ExecResult<&Self::PolicyType> { Ok(&self.sender_policies) }
}

pub trait HasChainMetadata {
    fn slot_time(&self) -> ExecResult<SlotTime>;
    fn tx_hash(&self) -> ExecResult<String>;
    fn height(&self) -> ExecResult<u64>;
}

impl HasChainMetadata for ChainMetadata {
    fn slot_time(&self) -> ExecResult<SlotTime> { Ok(self.slot_time) }
    fn tx_hash(&self) -> ExecResult<String> { Ok(self.tx_hash.clone()) }
    fn height(&self) -> ExecResult<u64> { Ok(self.height) }
}

fn call_common<C: HasCommon>(
    host: &mut C,
    f: CommonFunc,
    memory: &mut Vec<u8>,
    stack: &mut machine::RuntimeStack,
) -> machine::RunResult<()> {
    match f {
        CommonFunc::GetParameterSize => {
            // the cost of this function is adequately reflected by the base cost of a
            // function call so we do not charge extra.
            stack.push_value(host.param().len() as u32);
        }
        CommonFunc::GetParameterSection => {
            let offset = unsafe { stack.pop_u32() } as usize;
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_from_host_cost(length))?;
            let write_end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(write_end <= memory.len(), "Illegal memory access.");
            let end = std::cmp::min(offset + length as usize, host.param().len());
            ensure!(offset <= end, "Attempting to read non-existent parameter.");
            let amt = (&mut memory[start..write_end]).write(&host.param()[offset..end])?;
            stack.push_value(amt as u32);
        }
        CommonFunc::GetPolicySection => {
            let offset = unsafe { stack.pop_u32() } as usize;
            let length = unsafe { stack.pop_u32() };
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_from_host_cost(length))?;
            let start = unsafe { stack.pop_u32() } as usize;
            let write_end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(write_end <= memory.len(), "Illegal memory access.");
            let policies = host.policies()?.policies_to_bytes();
            let policies_bytes = policies.as_ref();
            let end = std::cmp::min(offset + length as usize, policies_bytes.len());
            ensure!(offset <= end, "Attempting to read non-existent policy.");
            let amt = (&mut memory[start..write_end]).write(&policies_bytes[offset..end])?;
            stack.push_value(amt as u32);
        }
        CommonFunc::LogEvent => {
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            let end = start + length as usize;
            ensure!(end <= memory.len(), "Illegal memory access.");
            if length <= constants::MAX_LOG_SIZE {
                // only charge if we actually log something.
                host.energy().tick_energy(log_event_cost(length))?;
                stack.push_value(host.logs().log_event(memory[start..end].to_vec()))
            } else {
                // otherwise the cost is adequately reflected by just the cost of a function
                // call.
                stack.push_value(-1i32)
            }
        }
        CommonFunc::LoadState => {
            let offset = unsafe { stack.pop_u32() };
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_from_host_cost(length))?;
            let end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(end <= memory.len(), "Illegal memory access.");
            let res = host.state().load_state(offset, &mut memory[start..end])?;
            stack.push_value(res);
        }
        CommonFunc::WriteState => {
            let offset = unsafe { stack.pop_u32() };
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_to_host_cost(length))?;
            let end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(end <= memory.len(), "Illegal memory access.");
            let res = host.state().write_state(offset, &memory[start..end])?;
            stack.push_value(res);
        }
        CommonFunc::ResizeState => {
            let new_size = stack.pop();
            let new_size = unsafe { new_size.short } as u32;
            let old_size = host.state().len();
            if new_size > old_size {
                // resizing is very similar to writing 0 to the newly allocated parts,
                // but since we don't have to read anything we charge it more cheaply.
                host.energy().tick_energy(additional_state_size_cost(new_size - old_size))?;
            }
            stack.push_value(host.state().resize_state(new_size));
        }
        CommonFunc::StateSize => {
            // the cost of this function is adequately reflected by the base cost of a
            // function call so we do not charge extra.
            stack.push_value(host.state().len());
        }
        CommonFunc::GetSlotTime => {
            // the cost of this function is adequately reflected by the base cost of a
            // function call so we do not charge extra.
            stack.push_value(host.metadata().slot_time()?.timestamp_millis());
        }
        CommonFunc::GetTxHash => {
            let start = unsafe { stack.pop_u32() } as usize;
            let write_end = start+64;
            let amt = (&mut memory[start..write_end]).write(&host.metadata().tx_hash()?.as_bytes())?;
            stack.push_value(amt as u32);
        }
        CommonFunc::GetHeight => {
            // the cost of this function is adequately reflected by the base cost of a
            // function call so we do not charge extra.
            stack.push_value(host.metadata().height()?);
        }
        CommonFunc::GetResult => {
            let offset = unsafe { stack.pop_u32() };
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_to_host_cost(length))?;
            let end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(end <= memory.len(), "Illegal memory access.");
            let res = host.result().write_return(offset, &memory[start..end])?;
            stack.push_value(res);
        }
        CommonFunc::GetState => {
            let offset = unsafe { stack.pop_u32() } as usize;
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_from_host_cost(length))?;
            let end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(end <= memory.len(), "Illegal memory access.");
            let res = host.state().get_state(&mut memory[start..end])?;
            let write_end = offset  + res.len() as usize;
            let amt = (&mut memory[offset..write_end]).write(&res)?;
            stack.push_value(amt as u32);
        }
        CommonFunc::SetState => {
            let v_length = unsafe { stack.pop_u32() };
            let v_start = unsafe { stack.pop_u32() } as usize;
            let k_length = unsafe { stack.pop_u32() };
            let k_start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_to_host_cost(v_length))?;
            let k_end = k_start + k_length as usize; // this cannot overflow on 64-bit machines.
            let v_end = v_start + v_length as usize;
            ensure!(v_end <= memory.len(), "Illegal memory access.");
            ensure!(k_end <= memory.len(), "Illegal memory access.");
            let res = host.state().set_state(&memory[k_start..k_end], &memory[v_start..v_end])?;
            stack.push_value(res);
        }
        CommonFunc::ValidateVC => {
            let length = unsafe { stack.pop_u32() };
            let start = unsafe { stack.pop_u32() } as usize;
            // charge energy linearly in the amount of data written.
            host.energy().tick_energy(copy_to_host_cost(length))?;
            let end = start + length as usize; // this cannot overflow on 64-bit machines.
            ensure!(end <= memory.len(), "Illegal memory access.");

            let vc = std::str::from_utf8(&memory[start..end])?;
            let res = host.did().validate_vc(vc.to_string());
            stack.push_value(res);
        }
    }
    Ok(())
}

impl<'a, Ctx: HasInitContext> machine::Host<ProcessedImports> for InitHost<'a, Ctx> {
    #[cfg_attr(not(feature = "fuzz-coverage"), inline(always))]
    fn tick_initial_memory(&mut self, num_pages: u32) -> machine::RunResult<()> {
        self.energy.charge_memory_alloc(num_pages)
    }

    #[cfg_attr(not(feature = "fuzz-coverage"), inline)]
    fn call(
        &mut self,
        f: &ProcessedImports,
        memory: &mut Vec<u8>,
        stack: &mut machine::RuntimeStack,
    ) -> machine::RunResult<()> {
        match f.tag {
            ImportFunc::ChargeEnergy => {
                self.energy.tick_energy(unsafe { stack.pop_u64() })?;
            }
            ImportFunc::TrackCall => {
                if let Some(fr) = self.activation_frames.checked_sub(1) {
                    self.activation_frames = fr
                } else {
                    bail!("Too many nested functions.")
                }
            }
            ImportFunc::TrackReturn => self.activation_frames += 1,
            ImportFunc::ChargeMemoryAlloc => {
                self.energy.charge_memory_alloc(unsafe { stack.peek_u32() })?;
            }
            ImportFunc::Common(cf) => call_common(self, cf, memory, stack)?,
            ImportFunc::InitOnly(InitOnlyFunc::GetInitOrigin) => {
                let start = unsafe { stack.pop_u32() } as usize;
                ensure!(start + 42 <= memory.len(), "Illegal memory access for init origin.");
                (&mut memory[start..start + 42])
                    .write_all(self.init_ctx.init_origin()?.as_ref())?;
            }
            ImportFunc::ReceiveOnly(_) => {
                bail!("Not implemented for init {:#?}.", f);
            }
        }
        Ok(())
    }
}

impl<'a, Ctx> ReceiveHost<'a, Ctx>
where
    Ctx: HasReceiveContext,
{
    pub fn call_receive_only(
        &mut self,
        rof: ReceiveOnlyFunc,
        memory: &mut Vec<u8>,
        stack: &mut machine::RuntimeStack,
    ) -> ExecResult<()> {
        match rof {
            ReceiveOnlyFunc::Accept => {
                self.energy.tick_energy(constants::BASE_ACTION_COST)?;
                stack.push_value(self.outcomes.accept());
            }
            ReceiveOnlyFunc::SimpleTransfer => {
                self.energy.tick_energy(constants::BASE_ACTION_COST)?;
                let amount = unsafe { stack.pop_u64() };
                let addr_start = unsafe { stack.pop_u32() } as usize;
                // Overflow is not possible in the next line on 64-bit machines.
                ensure!(addr_start + 42 <= memory.len(), "Illegal memory access.");
                stack.push_value(
                    self.outcomes.simple_transfer(&memory[addr_start..addr_start + 42], amount)?,
                )
            }
            ReceiveOnlyFunc::Send => {
                // all `as usize` are safe on 64-bit systems since we are converging from a u32
                let parameter_len = unsafe { stack.pop_u32() };
                self.energy().tick_energy(action_send_cost(parameter_len))?;
                let parameter_start = unsafe { stack.pop_u32() } as usize;
                // Overflow is not possible in the next line on 64-bit machines.
                let parameter_end = parameter_start + parameter_len as usize;
                let amount = unsafe { stack.pop_u64() };
                let receive_name_len = unsafe { stack.pop_u32() } as usize;
                let receive_name_start = unsafe { stack.pop_u32() } as usize;
                // Overflow is not possible in the next line on 64-bit machines.
                let receive_name_end = receive_name_start + receive_name_len;
                let addr_index = unsafe { stack.pop_u32() } as usize;
                ensure!(parameter_end <= memory.len(), "Illegal memory access.");
                ensure!(receive_name_end <= memory.len(), "Illegal memory access.");
                let res = self.outcomes.send(
                    &memory[addr_index..addr_index+42],
                    &memory[receive_name_start..receive_name_end],
                    amount,
                    &memory[parameter_start..parameter_end],
                )?;
                stack.push_value(res);
            }
            ReceiveOnlyFunc::CombineAnd => {
                self.energy.tick_energy(constants::BASE_ACTION_COST)?;
                let right = unsafe { stack.pop_u32() };
                let left = unsafe { stack.pop_u32() };
                let res = self.outcomes.combine_and(left, right)?;
                stack.push_value(res);
            }
            ReceiveOnlyFunc::CombineOr => {
                self.energy.tick_energy(constants::BASE_ACTION_COST)?;
                let right = unsafe { stack.pop_u32() };
                let left = unsafe { stack.pop_u32() };
                let res = self.outcomes.combine_or(left, right)?;
                stack.push_value(res);
            }
            ReceiveOnlyFunc::GetReceiveInvoker => {
                let start = unsafe { stack.pop_u32() } as usize;
                ensure!(start + 42 <= memory.len(), "Illegal memory access for receive invoker.");
                (&mut memory[start..start + 42]).write_all(self.receive_ctx.invoker()?.as_ref())?;
            }
            ReceiveOnlyFunc::GetReceiveSelfAddress => {
                // let start = unsafe { stack.pop_u32() } as usize;
                // ensure!(start + 16 <= memory.len(), "Illegal memory access for receive owner.");
                // (&mut memory[start..start + 8])
                //     .write_all(&self.receive_ctx.self_address()?.index.to_le_bytes())?;
                // (&mut memory[start + 8..start + 16])
                //     .write_all(&self.receive_ctx.self_address()?.subindex.to_le_bytes())?;
                let start = unsafe { stack.pop_u32() } as usize;
                ensure!(start + 42 <= memory.len(), "Illegal memory access for receive owner.");
                (&mut memory[start..start + 42]).write_all(self.receive_ctx.invoker()?.as_ref())?;
            }
            ReceiveOnlyFunc::GetReceiveSelfBalance => {
                stack.push_value(self.receive_ctx.self_balance()?.micro_gtu);
            }
            ReceiveOnlyFunc::GetReceiveSender => {
                let start = unsafe { stack.pop_u32() } as usize;
                ensure!(start < memory.len(), "Illegal memory access for receive sender.");
                self.receive_ctx
                    .sender()?
                    .serial::<&mut [u8]>(&mut &mut memory[start..])
                    .map_err(|_| anyhow!("Memory out of bounds."))?;
            }
            ReceiveOnlyFunc::GetReceiveOwner => {
                let start = unsafe { stack.pop_u32() } as usize;
                ensure!(start + 42 <= memory.len(), "Illegal memory access for receive owner.");
                (&mut memory[start..start + 42]).write_all(self.receive_ctx.owner()?.as_ref())?;
            }
        }
        Ok(())
    }
}

impl<'a, Ctx: HasReceiveContext> machine::Host<ProcessedImports> for ReceiveHost<'a, Ctx> {
    #[cfg_attr(not(feature = "fuzz-coverage"), inline(always))]
    fn tick_initial_memory(&mut self, num_pages: u32) -> machine::RunResult<()> {
        self.energy.charge_memory_alloc(num_pages)
    }

    #[cfg_attr(not(feature = "fuzz-coverage"), inline)]
    fn call(
        &mut self,
        f: &ProcessedImports,
        memory: &mut Vec<u8>,
        stack: &mut machine::RuntimeStack,
    ) -> machine::RunResult<()> {
        match f.tag {
            ImportFunc::ChargeEnergy => {
                let amount = unsafe { stack.pop_u64() };
                self.energy.tick_energy(amount)?;
            }
            ImportFunc::TrackCall => {
                if let Some(fr) = self.activation_frames.checked_sub(1) {
                    self.activation_frames = fr
                } else {
                    bail!("Too many nested functions.")
                }
            }
            ImportFunc::TrackReturn => self.activation_frames += 1,
            ImportFunc::ChargeMemoryAlloc => {
                self.energy.charge_memory_alloc(unsafe { stack.peek_u32() })?
            }
            ImportFunc::Common(cf) => call_common(self, cf, memory, stack)?,
            ImportFunc::ReceiveOnly(cro) => self.call_receive_only(cro, memory, stack)?,
            ImportFunc::InitOnly(InitOnlyFunc::GetInitOrigin) => {
                bail!("Not implemented for receive.");
            }
        }
        Ok(())
    }
}

pub type Parameter<'a> = &'a [u8];

pub type PolicyBytes<'a> = &'a [u8];

/// Invokes an init-function from a given artifact
pub fn invoke_init<C: RunnableCode, Ctx: HasInitContext>(
    artifact: &Artifact<ProcessedImports, C>,
    amount: u64,
    init_ctx: Ctx,
    init_name: &str,
    param: Parameter,
    energy: u64,
) -> ExecResult<InitResult> {
    let mut host = InitHost {
        energy: Energy {
            energy,
        },
        activation_frames: MAX_ACTIVATION_FRAMES,
        logs: Logs::new(),
        state: State::new(None),
        param,
        init_ctx: &init_ctx,
        result:   ReturnData::new(None),
    };

    let res = match artifact.run(&mut host, init_name, &[Value::I64(amount as i64)]) {
        Ok((res, _)) => res,
        Err(e) => {
            if e.downcast_ref::<OutOfEnergy>().is_some() {
                return Ok(InitResult::OutOfEnergy);
            } else {
                return Err(e);
            }
        }
    };
    let remaining_energy = host.energy.energy;
    // process the return value.
    // - 0 indicates success
    // - positive values are a protocol violation, so they lead to a runtime error
    // - negative values lead to a rejection with a specific reject reason.
    if let Some(Value::I32(n)) = res {
        if n == 0 {
            Ok(InitResult::Success {
                logs: host.logs,
                state: host.state,
                remaining_energy,
            })
        } else {
            Ok(InitResult::Reject {
                reason: reason_from_wasm_error_code(n)?,
                remaining_energy,
            })
        }
    } else {
        bail!("Wasm module should return a value.")
    }
}

/// Invokes an init-function from a given artifact *bytes*
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_init_from_artifact<Ctx: HasInitContext>(
    artifact_bytes: &[u8],
    amount: u64,
    init_ctx: Ctx,
    init_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<InitResult> {
    let artifact = utils::parse_artifact(artifact_bytes)?;
    invoke_init(&artifact, amount, init_ctx, init_name, parameter, energy)
}

/// Invokes an init-function from Wasm module bytes
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_init_from_source<Ctx: HasInitContext>(
    source_bytes: &[u8],
    amount: u64,
    init_ctx: Ctx,
    init_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<InitResult> {
    let artifact = utils::instantiate(&ConcordiumAllowedImports, source_bytes)?;
    invoke_init(&artifact, amount, init_ctx, init_name, parameter, energy)
}

/// Same as `invoke_init_from_source`, except that the module has cost
/// accounting instructions inserted before the init function is called.
/// metering.
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_init_with_metering_from_source<Ctx: HasInitContext>(
    source_bytes: &[u8],
    amount: u64,
    init_ctx: Ctx,
    init_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<InitResult> {
    let artifact = utils::instantiate_with_metering(&ConcordiumAllowedImports, source_bytes)?;
    invoke_init(&artifact, amount, init_ctx, init_name, parameter, energy)
}

/// Invokes an receive-function from a given artifact
pub fn invoke_receive<C: RunnableCode, Ctx: HasReceiveContext>(
    artifact: &Artifact<ProcessedImports, C>,
    amount: u64,
    receive_ctx: Ctx,
    current_state: &[u8],
    receive_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<ReceiveResult> {
    let mut host = ReceiveHost {
        energy:            Energy {
            energy,
        },
        activation_frames: MAX_ACTIVATION_FRAMES,
        logs:              Logs::new(),
        state:             State::new(Some(current_state)),
        param:             &parameter,
        receive_ctx:       &receive_ctx,
        outcomes:          Outcome::new(),
        result:            ReturnData::new(None),
    };

    let res = match artifact.run(&mut host, receive_name, &[Value::I64(amount as i64)]) {
        Ok((res, _)) => res,
        Err(e) => {
            if e.downcast_ref::<OutOfEnergy>().is_some() {
                return Ok(ReceiveResult::OutOfEnergy);
            } else {
                return Err(e);
            }
        }
    };
    let remaining_energy = host.energy.energy;
    if let Some(Value::I32(n)) = res {
        // FIXME: We should filter out to only return the ones reachable from
        // the root.
        let mut actions = host.outcomes.cur_state;
        if n >= 0 && (n as usize) < actions.len() {
            let n = n as usize;
            actions.truncate(n + 1);
            Ok(ReceiveResult::Success {
                logs: host.logs,
                state: host.state,
                actions,
                returndata:host.result,
                remaining_energy,
            })
        } else if n >= 0 {
            bail!("Invalid return.")
        } else {
            Ok(ReceiveResult::Reject {
                reason: reason_from_wasm_error_code(n)?,
                remaining_energy,
            })
        }
    } else {
        bail!(
            "Invalid return. Expected a value, but receive nothing. This should not happen for \
             well-formed modules"
        );
    }
}

/// Returns the passed Wasm error code if it is negative.
/// This function should only be called on negative numbers.
fn reason_from_wasm_error_code(n: i32) -> ExecResult<i32> {
    ensure!(
        n < 0,
        "Wasm return value of {} is treated as an error. Only negative should be treated as error.",
        n
    );
    Ok(n)
}

/// A helper trait to support invoking contracts when the policy is given as a
/// byte array, as well as when it is given in structured form, such as
/// Vec<OwnedPolicy>.
pub trait SerialPolicies<R: AsRef<[u8]>> {
    fn policies_to_bytes(&self) -> R;
}

impl<'a> SerialPolicies<&'a [u8]> for &'a [u8] {
    fn policies_to_bytes(&self) -> &'a [u8] { self }
}

impl SerialPolicies<Vec<u8>> for Vec<OwnedPolicy> {
    fn policies_to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let len = self.len() as u16;
        len.serial(&mut out).expect("Cannot fail writing to vec.");
        for policy in self.iter() {
            let bytes = to_bytes(policy);
            let internal_len = bytes.len() as u16;
            internal_len.serial(&mut out).expect("Cannot fail writing to vec.");
            out.extend_from_slice(&bytes);
        }
        out
    }
}

/// Invokes an receive-function from a given artifact *bytes*
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_receive_from_artifact<Ctx: HasReceiveContext>(
    artifact_bytes: &[u8],
    amount: u64,
    receive_ctx: Ctx,
    current_state: &[u8],
    receive_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<ReceiveResult> {
    let artifact = utils::parse_artifact(artifact_bytes)?;
    invoke_receive(&artifact, amount, receive_ctx, current_state, receive_name, parameter, energy)
}

/// Invokes an receive-function from Wasm module bytes
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_receive_from_source<Ctx: HasReceiveContext>(
    source_bytes: &[u8],
    amount: u64,
    receive_ctx: Ctx,
    current_state: &[u8],
    receive_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<ReceiveResult> {
    let artifact = utils::instantiate(&ConcordiumAllowedImports, source_bytes)?;
    invoke_receive(&artifact, amount, receive_ctx, current_state, receive_name, parameter, energy)
}

/// Invokes an receive-function from Wasm module bytes, injects the module with
/// metering.
#[cfg_attr(not(feature = "fuzz-coverage"), inline)]
pub fn invoke_receive_with_metering_from_source<Ctx: HasReceiveContext>(
    source_bytes: &[u8],
    amount: u64,
    receive_ctx: Ctx,
    current_state: &[u8],
    receive_name: &str,
    parameter: Parameter,
    energy: u64,
) -> ExecResult<ReceiveResult> {
    let artifact = utils::instantiate_with_metering(&ConcordiumAllowedImports, source_bytes)?;
    invoke_receive(&artifact, amount, receive_ctx, current_state, receive_name, parameter, energy)
}

/// A host which traps for any function call.
pub struct TrapHost;

impl<I> machine::Host<I> for TrapHost {
    fn tick_initial_memory(&mut self, _num_pages: u32) -> machine::RunResult<()> { Ok(()) }

    fn call(
        &mut self,
        _f: &I,
        _memory: &mut Vec<u8>,
        _stack: &mut machine::RuntimeStack,
    ) -> machine::RunResult<()> {
        bail!("TrapHost traps on all host calls.")
    }
}

/// A host which traps for any function call apart from `report_error` which it
/// prints to standard out.
pub struct TestHost;

impl validate::ValidateImportExport for TestHost {
    /// Simply ensure that there are no duplicates.
    #[cfg_attr(not(feature = "fuzz-coverage"), inline(always))]
    fn validate_import_function(
        &self,
        duplicate: bool,
        _mod_name: &Name,
        _item_name: &Name,
        _ty: &wasm_transform::types::FunctionType,
    ) -> bool {
        !duplicate
    }

    #[cfg_attr(not(feature = "fuzz-coverage"), inline(always))]
    fn validate_export_function(
        &self,
        _item_name: &Name,
        _ty: &wasm_transform::types::FunctionType,
    ) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
/// An auxiliary datatype used by `report_error` to be able to
/// retain the structured information in case we want to use it later
/// to insert proper links to the file, or other formatting.
pub enum ReportError {
    /// An error reported by `report_error`
    Reported {
        filename: String,
        line:     u32,
        column:   u32,
        msg:      String,
    },
    /// Some other source of error. We only have the description, and no
    /// location.
    Other {
        msg: String,
    },
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::Reported {
                filename,
                line,
                column,
                msg,
            } => write!(f, "{}, {}:{}:{}", msg, filename, line, column),
            ReportError::Other {
                msg,
            } => msg.fmt(f),
        }
    }
}

impl machine::Host<ArtifactNamedImport> for TestHost {
    fn tick_initial_memory(&mut self, _num_pages: u32) -> machine::RunResult<()> {
        // The test host does not count energy.
        Ok(())
    }

    fn call(
        &mut self,
        f: &ArtifactNamedImport,
        memory: &mut Vec<u8>,
        stack: &mut machine::RuntimeStack,
    ) -> machine::RunResult<()> {
        if f.matches("concordium", "report_error") {
            let column = unsafe { stack.pop_u32() };
            let line = unsafe { stack.pop_u32() };
            let filename_length = unsafe { stack.pop_u32() } as usize;
            let filename_start = unsafe { stack.pop_u32() } as usize;
            let msg_length = unsafe { stack.pop_u32() } as usize;
            let msg_start = unsafe { stack.pop_u32() } as usize;
            ensure!(filename_start + filename_length <= memory.len(), "Illegal memory access.");
            ensure!(msg_start + msg_length <= memory.len(), "Illegal memory access.");
            let msg = std::str::from_utf8(&memory[msg_start..msg_start + msg_length])?.to_owned();
            let filename =
                std::str::from_utf8(&memory[filename_start..filename_start + filename_length])?
                    .to_owned();
            bail!(ReportError::Reported {
                filename,
                line,
                column,
                msg
            })
        } else {
            bail!("Unsupported host function call.")
        }
    }
}

/// Instantiates the module with an external function to report back errors.
/// Then tries to run exported test-functions, which are present if compile with
/// the wasm-test feature.
///
/// The return value is a list of pairs (test_name, result)
/// The result is None if the test passed, or an error message
/// if it failed. The error message is the one reported to by report_error, or
/// some internal invariant violation.
pub fn run_module_tests(module_bytes: &[u8]) -> ExecResult<Vec<(String, Option<ReportError>)>> {
    let artifact = utils::instantiate::<ArtifactNamedImport, _>(&TestHost, module_bytes)?;
    let mut out = Vec::with_capacity(artifact.export.len());
    for name in artifact.export.keys() {
        if let Some(test_name) = name.as_ref().strip_prefix("concordium_test ") {
            let res = artifact.run(&mut TestHost, name, &[]);
            match res {
                Ok(_) => out.push((test_name.to_owned(), None)),
                Err(msg) => {
                    if let Some(err) = msg.downcast_ref::<ReportError>() {
                        out.push((test_name.to_owned(), Some(err.clone())));
                    } else {
                        out.push((
                            test_name.to_owned(),
                            Some(ReportError::Other {
                                msg: msg.to_string(),
                            }),
                        ))
                    }
                }
            };
        }
    }
    Ok(out)
}

/// Tries to generate a state schema and schemas for parameters of methods.
pub fn generate_contract_schema(module_bytes: &[u8]) -> ExecResult<schema::Module> {
    let artifact = utils::instantiate::<ArtifactNamedImport, _>(&TestHost, module_bytes)?;

    let mut contract_schemas = BTreeMap::new();

    for name in artifact.export.keys() {
        if let Some(contract_name) = name.as_ref().strip_prefix("concordium_schema_state_") {
            let schema_type = generate_schema_run(&artifact, name.as_ref())?;

            // Get the mutable reference to the contract schema, or make a new empty one if
            // an entry does not yet exist.
            let contract_schema = contract_schemas
                .entry(contract_name.to_owned())
                .or_insert_with(schema::Contract::empty);

            contract_schema.state = Some(schema_type);
        } else if let Some(contract_name) = name.as_ref().strip_prefix("concordium_schema_event_") {  //xq log-event
            let schema_type = generate_schema_run(&artifact, name.as_ref())?;

            // Get the mutable reference to the contract schema, or make a new empty one if
            // an entry does not yet exist.
            let contract_schema = contract_schemas
                .entry(contract_name.to_owned())
                .or_insert_with(schema::Contract::empty);

            contract_schema.event = Some(schema_type);
        }
        else if let Some(rest) = name.as_ref().strip_prefix("concordium_schema_function_") {
            if let Some(contract_name) = rest.strip_prefix("init_") {
                let schema_type = generate_schema_run(&artifact, name.as_ref())?;

                let contract_schema = contract_schemas
                    .entry(contract_name.to_owned())
                    .or_insert_with(schema::Contract::empty);
                contract_schema.init = Some(schema_type);
            } else if let Some(contract_func_name) = rest.strip_prefix("result_") {  //xuqiang json result
                let schema_type = generate_schema_run(&artifact, name.as_ref())?;

                // Generates receive-function parameter schema type
                let split_name: Vec<_> = contract_func_name.splitn(2, '.').collect();
                let contract_name = split_name[0];
                let function_name = split_name[1];

                let contract_schema = contract_schemas
                    .entry(contract_name.to_owned())
                    .or_insert_with(schema::Contract::empty);
                let mut name = function_name.to_owned();
                name.insert_str(0, "result_");
                contract_schema.receive.insert(name, schema_type);
            } else if rest.contains('.') {
                let schema_type = generate_schema_run(&artifact, name.as_ref())?;

                // Generates receive-function parameter schema type
                let split_name: Vec<_> = rest.splitn(2, '.').collect();
                let contract_name = split_name[0];
                let function_name = split_name[1];

                let contract_schema = contract_schemas
                    .entry(contract_name.to_owned())
                    .or_insert_with(schema::Contract::empty);

                contract_schema.receive.insert(function_name.to_owned(), schema_type);
            } else {
                // do nothing, some other function that is neither init nor
                // receive.
            }
        }
    }

    Ok(schema::Module {
        contracts: contract_schemas,
    })
}

/// Runs the given schema function and reads the resulting schema from memory,
/// attempting to parse it as a type. If this fails, an error is returned.
fn generate_schema_run<I: TryFromImport, C: RunnableCode>(
    artifact: &Artifact<I, C>,
    schema_fn_name: &str,
) -> ExecResult<schema::Type> {
    let (ptr, memory) = if let (Some(Value::I32(ptr)), memory) =
        artifact.run(&mut TrapHost, schema_fn_name, &[])?
    {
        (ptr as u32 as usize, memory)
    } else {
        bail!("Schema derivation function malformed.")
    };

    // First we read an u32 which is the length of the serialized schema
    ensure!(ptr + 4 <= memory.len(), "Illegal memory access.");
    let len = u32::deserial(&mut Cursor::new(&memory[ptr..ptr + 4]))
        .map_err(|_| anyhow!("Cannot read schema length."))?;

    // Read the schema with offset of the u32
    ensure!(ptr + 4 + len as usize <= memory.len(), "Illegal memory access when reading schema.");
    let schema_bytes = &memory[ptr + 4..ptr + 4 + len as usize];
    schema::Type::deserial(&mut Cursor::new(schema_bytes))
        .map_err(|_| anyhow!("Failed deserialising the schema."))
}

/// Get the init methods of the module.
pub fn get_inits(module: &Module) -> Vec<&Name> {
    let mut out = Vec::new();
    for export in module.export.exports.iter() {
        if export.name.as_ref().starts_with("init_") && !export.name.as_ref().contains('.') {
            if let ExportDescription::Func {
                ..
            } = export.description
            {
                out.push(&export.name);
            }
        }
    }
    out
}

/// Get the receive methods of the module.
pub fn get_receives(module: &Module) -> Vec<&Name> {
    let mut out = Vec::new();
    for export in module.export.exports.iter() {
        if export.name.as_ref().contains('.') {
            if let ExportDescription::Func {
                ..
            } = export.description
            {
                out.push(&export.name);
            }
        }
    }
    out
}

/// Get the embedded schema if it exists
pub fn get_embedded_schema(bytes: &[u8]) -> ExecResult<schema::Module> {
    let skeleton = parse_skeleton(bytes)?;
    let mut schema_sections = Vec::new();
    for ucs in skeleton.custom.iter() {
        let cs = parse_custom(ucs)?;
        if cs.name.as_ref() == "concordium-schema-v1" {
            schema_sections.push(cs)
        }
    }
    let section =
        schema_sections.first().ok_or_else(|| anyhow!("No schema found in the module"))?;
    let source = &mut Cursor::new(section.contents);
    source.get().map_err(|_| anyhow!("Failed parsing schema"))
}
