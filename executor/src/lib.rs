pub mod wasmtime;
pub mod wasmtime2;
pub mod types;
pub mod exec;
pub mod utils;
pub mod schema_json;
pub mod gas;
pub mod geecowasm;
pub mod jvm;
use crate::{
    types::{ ContractResult, ContractError},
};
pub use concordium_contracts_common;

pub trait Contract{
    fn exec(&mut self, binary:Option<&[u8]>, amount:i64) -> Result<ContractResult, ContractError>;
}

pub trait VM{
    fn run(&self, code:&[u8], amount:i64) -> Result<ContractResult, ContractError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
