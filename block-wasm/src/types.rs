
use std::collections::BTreeMap;

//use anyhow::Ok;
use core::result::Result::Ok;
use block_executor::{
    errors::Error,
    executor::{BlockExecutor, RAYON_EXEC_POOL, MVHashMapView},
    task::{Transaction, TransactionOutput, ExecutorTask, ExecutionStatus}
};
use executor::{
    wasmtime2::{execute_transaction}, types::{ContextStm, WasmTransactionOutput, ExecKind},
    };
use wasm_chain_integration::{ReturnData, Logs};
use aptos_types::{
    state_store::state_key::StateKey, 
    write_set::{WriteOp, WriteSet}, 
    vm_status::StatusCode, transaction::{TransactionStatus, ExecutionStatus as TxStatus},
};
use aptos_state_view::StateView;
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
    vm_status::VMStatus,
};
use concordium_contracts_common::{AccountAddress};
use aptos_aggregator::delta_change_set::{DeltaChangeSet, DeltaOp};

#[derive(Clone)]
pub struct PreprocessedTransaction {
    pub kind: ExecKind,
    pub sender: AccountAddress,
    pub dest: AccountAddress,
    pub contract_name:String,
    pub func_name: String,
    pub balance:u64,
    pub owner:AccountAddress,
    pub gas_limit:u64,
    pub gas:bool,
    pub code: Vec<u8>,
    pub param: Vec<u8>,
    pub amount: u64,
    pub index:u64,
    pub state: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct GeeTransactionOutput {
    pub data: ReturnData,
    write_set: WriteSet,
    events: Logs,
    gas_used: u64,
    status: TransactionStatus,
}

#[derive(Clone, Debug)]
pub struct TransactionOutputExt {
    delta_change_set: DeltaChangeSet,
    output: GeeTransactionOutput,
}

pub(crate) struct GeecoTransactionOutput(TransactionOutputExt);

impl GeecoTransactionOutput {
    pub fn new(output: TransactionOutputExt) -> Self {
        Self(output)
    }

    pub fn into(self) -> TransactionOutputExt {
        self.0
    }

    pub fn as_ref(&self) -> &TransactionOutputExt {
        &self.0
    }
}


impl Transaction for PreprocessedTransaction {
    type Key = StateKey;
    type Value = WriteOp;
}


impl TransactionOutput for GeecoTransactionOutput {
    type T = PreprocessedTransaction;

    fn get_writes(&self) -> Vec<(StateKey, WriteOp)> {
        self.0
            .output
            .write_set
            .iter()
            .map(|(key, op)| (key.clone(), op.clone()))
            .collect()
    }

    fn get_deltas(&self) -> Vec<(StateKey, u8)> {
        todo!()
    }

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self {
        Self(TransactionOutputExt{
            delta_change_set: DeltaChangeSet::empty(),
            output: GeeTransactionOutput{            
                data:ReturnData::new(None),
                write_set:WriteSet::default(),
                events:Logs::new(),
                gas_used:0,
                status:TransactionStatus::Retry,}
            }
        )
    }
}






pub(crate) struct GeecoExecutorTask<'a, S>{
    _view: &'a S,
}


impl<'a, S: 'a + StateView> ExecutorTask for GeecoExecutorTask<'a, S> {
    type T = PreprocessedTransaction;
    type Output = GeecoTransactionOutput;
    type Error = VMStatus;
    type Argument = &'a S;

    fn init(argument: &'a S) -> Self {
        GeecoExecutorTask{
            _view:argument
        }
    }

    fn execute_transaction_btree_view(
        &self,
        view: &BTreeMap<StateKey, WriteOp>,
        txn: &PreprocessedTransaction,
        txn_idx: usize,
    ) -> ExecutionStatus<GeecoTransactionOutput, VMStatus> {
        todo!()
    }

    fn execute_transaction_mvhashmap_view(
        &self,
        view: &MVHashMapView<StateKey, WriteOp>,
        txn: &PreprocessedTransaction,
    ) -> ExecutionStatus<GeecoTransactionOutput, VMStatus> {
        let context:ContextStm<StateKey> = ContextStm::new_call(
            txn.kind.clone(),
            &txn.contract_name,
            &txn.func_name,
            &txn.param,
            concordium_contracts_common::Amount { micro_gtu: txn.balance },
            txn.sender,
            txn.owner,
            txn.dest,
            txn.gas_limit,
            txn.gas,
            view,
            &txn.state,
        );
        match execute_transaction(&txn.func_name, context, &txn.code, 0){
            Ok(o) => match o{
                WasmTransactionOutput::Success { state, logs, actions, returndata, remaining_gas , write_set } => {
                    ExecutionStatus::Success(GeecoTransactionOutput(TransactionOutputExt{
                        delta_change_set: DeltaChangeSet::empty(),
                        output: GeeTransactionOutput {
                            data: returndata,
                            write_set,
                            events: logs,
                            gas_used: remaining_gas,
                            status: TransactionStatus::Keep(TxStatus::Success),
                        }
                        }
                    ))
                },
                //WasmTransactionOutput::Reject { reason, remaining_gas } => return ExecutionStatus::Abort(VMStatus::Error(StatusCode::UNKNOWN_VALIDATION_STATUS)),
                WasmTransactionOutput::Reject { reason, remaining_gas } =>{
                    ExecutionStatus::Success(GeecoTransactionOutput(TransactionOutputExt{
                        delta_change_set: DeltaChangeSet::empty(),
                        output: GeeTransactionOutput {
                            data: ReturnData::new(None),
                            write_set:Default::default(),
                            events: Logs::new(),
                            gas_used: remaining_gas,
                            status: TransactionStatus::Keep(TxStatus::MiscellaneousError(None)),
                        }
                        }
                    ))
                },
            },
            Err(e) => {
                println!("execute_transaction:{}",e);
                return ExecutionStatus::Abort(VMStatus::Error(StatusCode::UNKNOWN_VALIDATION_STATUS))
            },
        }
    }
}

pub struct MockStateView();

impl StateView for MockStateView{
    fn get_state_value(&self, _state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn is_genesis(&self) -> bool {
        false
    }

    fn get_usage(&self) -> anyhow::Result<aptos_types::state_store::state_storage_usage::StateStorageUsage> {
        Ok(aptos_types::state_store::state_storage_usage::StateStorageUsage::Untracked)
    }
}

pub struct BlockGeecoVM();

impl BlockGeecoVM {

    pub fn new()->Self{
        Self()
    }

    pub fn execute_block<S: StateView>(
        transactions: Vec<PreprocessedTransaction>,
        state_view: &S,
        concurrency_level: usize,
    ) -> Result<Vec<GeeTransactionOutput>, VMStatus> {

        let executor =
            BlockExecutor::<PreprocessedTransaction, GeecoExecutorTask<S>>::new(concurrency_level);

        // let mut ret = if concurrency_level > 1 {
        //     executor
        //         .execute_transactions_parallel(state_view, &transactions)
        //         .map(|(results)| {
        //             results.0
        //         })
        // } else {
        //     executor
        //         .execute_transactions_sequential(state_view, &transactions)
        //         .map(Self::process_sequential_block_output)
        // };

        let mut ret = executor
        .execute_transactions_parallel(state_view, &transactions)
        .map(|(results)| {
            results.0
        });

        // if ret == Err(Error::ModulePathReadWrite) {
        //     //debug!("[Execution]: Module read & written, sequential fallback");

        //     ret = executor
        //         .execute_transactions_sequential(state_view, &transactions)
        //         .map(Self::process_sequential_block_output);
        // }

        // Explicit async drop. Happens here because we can't currently move to
        // BlockExecutor due to the Module publishing fallback. TODO: fix after
        // module publishing fallback is removed.
        RAYON_EXEC_POOL.spawn(move || {
            // Explicit async drops.
            drop(transactions);
        });

        match ret {
            Ok(outputs) => Ok(outputs.into_iter().map(|op| op.0.output).collect()),
            Err(Error::ModulePathReadWrite) => {
                unreachable!("[Execution]: Must be handled by sequential fallback")
            }
            Err(Error::UserError(err)) => Err(err),
        }
    }
}

