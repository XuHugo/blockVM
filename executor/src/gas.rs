use pwasm_utils::{self, rules};


/// Wasm cost table
#[derive(Debug)]
pub struct WasmCosts {
    /// Default opcode cost
    pub regular: u32,
    /// Div operations multiplier.
    pub div: u32,
    /// Div operations multiplier.
    pub mul: u32,
    /// Memory (load/store) operations multiplier.
    pub mem: u32,
    /// General static query of U256 value from env-info
    pub static_u256: u32,
    /// General static query of Address value from env-info
    pub static_address: u32,
    /// Memory stipend. Amount of free memory (in 64kb pages) each contract can use for stack.
    pub initial_mem: u32,
    /// Grow memory cost, per page (64kb)
    pub grow_mem: u32,
    /// Memory copy cost, per byte
    pub memcpy: u32,
    /// Max stack height (native WebAssembly stack limiter)
    pub max_stack_height: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` / `opcodes_div`
    pub opcodes_mul: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` / `opcodes_div`
    pub opcodes_div: u32,
    /// Whether create2 extern function is activated.
    pub have_create2: bool,
    /// Whether gasleft extern function is activated.
    pub have_gasleft: bool,
}


impl Default for WasmCosts {
    fn default() -> Self {
        WasmCosts {
            regular: 1,
            div: 16,
            mul: 4,
            mem: 2,
            static_u256: 64,
            static_address: 40,
            initial_mem: 4096, //4096
            grow_mem: 8192, //8192
            memcpy: 1,
            max_stack_height: 64 * 1024,
            opcodes_mul: 3,
            opcodes_div: 8,
            have_create2: false,
            have_gasleft: false,
        }
    }
}

pub fn gas_rules(wasm_costs: &WasmCosts) -> rules::Set {
    rules::Set::new(wasm_costs.regular, {
        let mut vals = ::std::collections::BTreeMap::new();
        vals.insert(
            rules::InstructionType::Load,
            rules::Metering::Fixed(wasm_costs.mem as u32),
        );
        vals.insert(
            rules::InstructionType::Store,
            rules::Metering::Fixed(wasm_costs.mem as u32),
        );
        vals.insert(
            rules::InstructionType::Div,
            rules::Metering::Fixed(wasm_costs.div as u32),
        );
        vals.insert(
            rules::InstructionType::Mul,
            rules::Metering::Fixed(wasm_costs.mul as u32),
        );
        vals
    })
        .with_grow_cost(wasm_costs.grow_mem)
        .with_forbidden_floats()
}

/// VM errors.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// `OutOfGas` is returned when transaction execution runs out of gas.
    /// The state should be reverted to the state from before the
    /// transaction execution. But it does not mean that transaction
    /// was invalid. Balance still should be transfered and nonce
    /// should be increased.
    OutOfGas,
    /// `BadJumpDestination` is returned when execution tried to move
    /// to position that wasn't marked with JUMPDEST instruction
    BadJumpDestination {
        /// Position the code tried to jump to.
        destination: usize,
    },
    /// `BadInstructions` is returned when given instruction is not supported
    BadInstruction {
        /// Unrecognized opcode
        instruction: u8,
    },
    /// `StackUnderflow` when there is not enough stack elements to execute instruction
    StackUnderflow {
        /// Invoked instruction
        instruction: &'static str,
        /// How many stack elements was requested by instruction
        wanted: usize,
        /// How many elements were on stack
        on_stack: usize,
    },
    /// When execution would exceed defined Stack Limit
    OutOfStack {
        /// Invoked instruction
        instruction: &'static str,
        /// How many stack elements instruction wanted to push
        wanted: usize,
        /// What was the stack limit
        limit: usize,
    },
    /// When there is not enough subroutine stack elements to return from
    SubStackUnderflow {
        /// How many stack elements was requested by instruction
        wanted: usize,
        /// How many elements were on stack
        on_stack: usize,
    },
    /// When execution would exceed defined subroutine Stack Limit
    OutOfSubStack {
        /// How many stack elements instruction wanted to pop
        wanted: usize,
        /// What was the stack limit
        limit: usize,
    },
    /// When the code walks into a subroutine, that is not allowed
    InvalidSubEntry,
    /// Built-in contract failed on given input
    BuiltIn(&'static str),
    /// When execution tries to modify the state in static context
    MutableCallInStaticContext,
    /// Invalid code to deploy as a contract
    InvalidCode,
    /// Likely to cause consensus issues.
    Internal(String),
    /// Wasm runtime error
    Wasm(String),
    /// Out of bounds access in RETURNDATACOPY.
    OutOfBounds,
    /// Execution has been reverted with REVERT.
    Reverted,
}



