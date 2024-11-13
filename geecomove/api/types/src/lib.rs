
mod move_types;
mod bytecode;
mod wrappers;
mod address;


// extern crate  move_binary_format;
// extern crate  move_core_types;
// extern crate  anyhow;
// extern crate  aptos_types;
// extern crate  serde;
// extern crate  move_resource_viewer;
// extern crate  poem_openapi;

pub use move_types::{
    verify_field_identifier, verify_function_identifier, verify_module_identifier, EntryFunctionId,
    HexEncodedBytes, MoveAbility, MoveFunction, MoveFunctionGenericTypeParam,
    MoveFunctionVisibility, MoveModule, MoveModuleBytecode, MoveModuleId, MoveResource,
    MoveScriptBytecode, MoveStruct, MoveStructField, MoveStructTag, MoveType, MoveValue,
    MAX_RECURSIVE_TYPES_ALLOWED, U128, U256, U64,
};
use serde::{Deserialize, Deserializer};
pub use bytecode::Bytecode;
use std::str::FromStr;
pub use wrappers::{EventGuid, IdentifierWrapper, StateKeyWrapper};
pub use address::Address;



pub fn deserialize_from_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    use serde::de::Error;

    let s = <String>::deserialize(deserializer)?;
    s.parse::<T>().map_err(D::Error::custom)
}

/// For verifying a given struct
pub trait VerifyInput {
    fn verify(&self) -> anyhow::Result<()>;
}

/// For verifying a given struct that needs to limit recursion
pub trait VerifyInputWithRecursion {
    fn verify(&self, recursion_count: u8) -> anyhow::Result<()>;
}
