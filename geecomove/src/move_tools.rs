use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use move_core_types::account_address::AccountAddress;
use move_core_types::transaction_argument::TransactionArgument;
use move_core_types::u256::U256;
use serde::Serialize;
use crate::common_ext::types::{CliError, CliTypedResult, load_account_arg};




#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FunctionArgType {
    Address,
    Bool,
    Hex,
    HexArray,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Raw,
    Vector(Box<FunctionArgType>),
}

impl Display for FunctionArgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionArgType::Address => write!(f, "address"),
            FunctionArgType::Bool => write!(f, "bool"),
            FunctionArgType::Hex => write!(f, "hex"),
            FunctionArgType::HexArray => write!(f, "hex_array"),
            FunctionArgType::String => write!(f, "string"),
            FunctionArgType::U8 => write!(f, "u8"),
            FunctionArgType::U16 => write!(f, "u16"),
            FunctionArgType::U32 => write!(f, "u32"),
            FunctionArgType::U64 => write!(f, "u64"),
            FunctionArgType::U128 => write!(f, "u128"),
            FunctionArgType::U256 => write!(f, "u256"),
            FunctionArgType::Raw => write!(f, "raw"),
            FunctionArgType::Vector(inner) => write!(f, "vector<{}>", inner),
        }
    }
}

impl FunctionArgType {
    fn parse_arg(&self, arg: &str) -> CliTypedResult<Vec<u8>> {
        match self {
            FunctionArgType::Address => bcs::to_bytes(
                &load_account_arg(arg)
                    .map_err(|err| CliError::UnableToParse("address", err.to_string()))?,
            ),
            FunctionArgType::Bool => bcs::to_bytes(
                &bool::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("bool", err.to_string()))?,
            ),
            FunctionArgType::Hex => bcs::to_bytes(
                &hex::decode(arg).map_err(|err| CliError::UnableToParse("hex", err.to_string()))?,
            ),
            FunctionArgType::HexArray => {
                let mut encoded = vec![];
                for sub_arg in arg.split(',') {
                    encoded.push(hex::decode(sub_arg).map_err(|err| {
                        CliError::UnableToParse(
                            "hex_array",
                            format!("Failed to parse hex array: {}", err),
                        )
                    })?);
                }
                bcs::to_bytes(&encoded)
            },
            FunctionArgType::String => bcs::to_bytes(arg),
            FunctionArgType::U8 => bcs::to_bytes(
                &u8::from_str(arg).map_err(|err| CliError::UnableToParse("u8", err.to_string()))?,
            ),
            FunctionArgType::U16 => bcs::to_bytes(
                &u16::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("u16", err.to_string()))?,
            ),
            FunctionArgType::U32 => bcs::to_bytes(
                &u32::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("u32", err.to_string()))?,
            ),
            FunctionArgType::U64 => bcs::to_bytes(
                &u64::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("u64", err.to_string()))?,
            ),
            FunctionArgType::U128 => bcs::to_bytes(
                &u128::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("u128", err.to_string()))?,
            ),
            FunctionArgType::U256 => bcs::to_bytes(
                &U256::from_str(arg)
                    .map_err(|err| CliError::UnableToParse("u256", err.to_string()))?,
            ),
            FunctionArgType::Raw => {
                let raw = hex::decode(arg)
                    .map_err(|err| CliError::UnableToParse("raw", err.to_string()))?;
                Ok(raw)
            },
            FunctionArgType::Vector(inner) => {
                let parsed = match inner.deref() {
                    FunctionArgType::Address => parse_vector_arg(arg, |arg| {
                        load_account_arg(arg).map_err(|err| {
                            CliError::UnableToParse("vector<address>", err.to_string())
                        })
                    }),
                    FunctionArgType::Bool => parse_vector_arg(arg, |arg| {
                        bool::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<bool>", err.to_string()))
                    }),
                    FunctionArgType::Hex => parse_vector_arg(arg, |arg| {
                        hex::decode(arg)
                            .map_err(|err| CliError::UnableToParse("vector<hex>", err.to_string()))
                    }),
                    FunctionArgType::U8 => parse_vector_arg(arg, |arg| {
                        u8::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u8>", err.to_string()))
                    }),
                    FunctionArgType::U16 => parse_vector_arg(arg, |arg| {
                        u16::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u16>", err.to_string()))
                    }),
                    FunctionArgType::U32 => parse_vector_arg(arg, |arg| {
                        u32::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u32>", err.to_string()))
                    }),
                    FunctionArgType::U64 => parse_vector_arg(arg, |arg| {
                        u64::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u64>", err.to_string()))
                    }),
                    FunctionArgType::U128 => parse_vector_arg(arg, |arg| {
                        u128::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u128>", err.to_string()))
                    }),
                    FunctionArgType::U256 => parse_vector_arg(arg, |arg| {
                        U256::from_str(arg)
                            .map_err(|err| CliError::UnableToParse("vector<u256>", err.to_string()))
                    }),
                    vector_type => {
                        panic!("Unsupported vector type vector<{}>", vector_type)
                    },
                }?;
                Ok(parsed)
            },
        }
            .map_err(|err| CliError::BCS("arg", err))
    }
}

fn parse_vector_arg<T: Serialize, F: Fn(&str) -> CliTypedResult<T>>(
    args: &str,
    parse: F,
) -> CliTypedResult<Vec<u8>> {
    let mut parsed_args = vec![];
    let args = args.split(',');
    for arg in args {
        parsed_args.push(parse(arg)?);
    }

    bcs::to_bytes(&parsed_args).map_err(|err| CliError::BCS("arg", err))
}
impl FromStr for FunctionArgType {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "address" => Ok(FunctionArgType::Address),
            "bool" => Ok(FunctionArgType::Bool),
            "hex" => Ok(FunctionArgType::Hex),
            "string" => Ok(FunctionArgType::String),
            "u8" => Ok(FunctionArgType::U8),
            "u16" => Ok(FunctionArgType::U16),
            "u32" => Ok(FunctionArgType::U32),
            "u64" => Ok(FunctionArgType::U64),
            "u128" => Ok(FunctionArgType::U128),
            "u256" => Ok(FunctionArgType::U256),
            "hex_array" => Ok(FunctionArgType::HexArray),
            "raw" => Ok(FunctionArgType::Raw),
            str => {
                // If it's a vector, go one level inside
                if str.starts_with("vector<") && str.ends_with('>') {
                    let arg = FunctionArgType::from_str(&str[7..str.len() - 1])?;

                    // String gets confusing on parsing by commas
                    if arg == FunctionArgType::String {
                        return Err(CliError::CommandArgumentError(
                            "vector<string> is not supported".to_string(),
                        ));
                    } else if arg == FunctionArgType::Raw {
                        return Err(CliError::CommandArgumentError(
                            "vector<raw> is not supported".to_string(),
                        ));
                    } else if matches!(arg, FunctionArgType::Vector(_)) {
                        return Err(CliError::CommandArgumentError(
                            "nested vector<vector<_>> is not supported".to_string(),
                        ));
                    } else if arg == FunctionArgType::HexArray {
                        return Err(CliError::CommandArgumentError(
                            "nested vector<hex_array> is not supported".to_string(),
                        ));
                    }

                    Ok(FunctionArgType::Vector(Box::new(arg)))
                } else {
                    Err(CliError::CommandArgumentError(format!("Invalid arg type '{}'.  Must be one of: ['address','bool','hex','hex_array','string','u8','u16','u32','u64','u128','u256','raw', 'vector<inner_type>']", str)))
                }
            },
        }
    }
}


fn txn_arg_parser<T: serde::de::DeserializeOwned>(
    data: &[u8],
    label: &'static str,
) -> Result<T, CliError> {
    bcs::from_bytes(data).map_err(|err| CliError::UnableToParse(label, err.to_string()))
}


/// A parseable arg with a type separated by a colon
pub struct ArgWithType {
    pub(crate) _ty: FunctionArgType,
    pub arg: Vec<u8>,
}

impl ArgWithType {
    pub fn address(account_address: AccountAddress) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Address,
            arg: bcs::to_bytes(&account_address).unwrap(),
        }
    }

    pub fn u64(arg: u64) -> Self {
        ArgWithType {
            _ty: FunctionArgType::U64,
            arg: bcs::to_bytes(&arg).unwrap(),
        }
    }

    pub fn bytes(arg: Vec<u8>) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Raw,
            arg: bcs::to_bytes(&arg).unwrap(),
        }
    }

    pub fn raw(arg: Vec<u8>) -> Self {
        ArgWithType {
            _ty: FunctionArgType::Raw,
            arg,
        }
    }

    pub fn to_json(&self) -> CliTypedResult<serde_json::Value> {
        match self._ty.clone() {
            FunctionArgType::Address => {
                serde_json::to_value(bcs::from_bytes::<AccountAddress>(&self.arg)?)
            },
            FunctionArgType::Bool => serde_json::to_value(bcs::from_bytes::<bool>(&self.arg)?),
            FunctionArgType::Hex => serde_json::to_value(bcs::from_bytes::<Vec<u8>>(&self.arg)?),
            FunctionArgType::String => serde_json::to_value(bcs::from_bytes::<String>(&self.arg)?),
            FunctionArgType::U8 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U16 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U32 => serde_json::to_value(bcs::from_bytes::<u32>(&self.arg)?),
            FunctionArgType::U64 => {
                serde_json::to_value(bcs::from_bytes::<u64>(&self.arg)?.to_string())
            },
            FunctionArgType::U128 => {
                serde_json::to_value(bcs::from_bytes::<u128>(&self.arg)?.to_string())
            },
            FunctionArgType::U256 => {
                serde_json::to_value(bcs::from_bytes::<U256>(&self.arg)?.to_string())
            },
            FunctionArgType::Raw => serde_json::to_value(&self.arg),
            FunctionArgType::HexArray => {
                serde_json::to_value(bcs::from_bytes::<Vec<Vec<u8>>>(&self.arg)?)
            },
            FunctionArgType::Vector(inner) => match inner.deref() {
                FunctionArgType::Address => {
                    serde_json::to_value(bcs::from_bytes::<Vec<AccountAddress>>(&self.arg)?)
                },
                FunctionArgType::Bool => {
                    serde_json::to_value(bcs::from_bytes::<Vec<bool>>(&self.arg)?)
                },
                FunctionArgType::Hex => {
                    serde_json::to_value(bcs::from_bytes::<Vec<Vec<u8>>>(&self.arg)?)
                },
                FunctionArgType::String => {
                    serde_json::to_value(bcs::from_bytes::<Vec<String>>(&self.arg)?)
                },
                FunctionArgType::U8 => serde_json::to_value(bcs::from_bytes::<Vec<u8>>(&self.arg)?),
                FunctionArgType::U16 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u16>>(&self.arg)?)
                },
                FunctionArgType::U32 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u32>>(&self.arg)?)
                },
                FunctionArgType::U64 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u64>>(&self.arg)?)
                },
                FunctionArgType::U128 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<u128>>(&self.arg)?)
                },
                FunctionArgType::U256 => {
                    serde_json::to_value(bcs::from_bytes::<Vec<U256>>(&self.arg)?)
                },
                FunctionArgType::Raw | FunctionArgType::HexArray | FunctionArgType::Vector(_) => {
                    return Err(CliError::UnexpectedError(
                        "Nested vectors not supported".to_string(),
                    ));
                },
            },
        }
            .map_err(|err| {
                CliError::UnexpectedError(format!("Failed to parse argument to JSON {}", err))
            })
    }
}

impl FromStr for ArgWithType {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Splits on the first colon, returning at most `2` elements
        // This is required to support args that contain a colon
        let parts: Vec<_> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(CliError::CommandArgumentError(
                "Arguments must be pairs of <type>:<arg> e.g. bool:true".to_string(),
            ));
        }

        let ty = FunctionArgType::from_str(parts.first().unwrap())?;
        let arg = parts.last().unwrap();
        let arg = ty.parse_arg(arg)?;

        Ok(ArgWithType { _ty: ty, arg })
    }
}

impl TryInto<TransactionArgument> for ArgWithType {
    type Error = CliError;

    fn try_into(self) -> Result<TransactionArgument, Self::Error> {
        match self._ty {
            FunctionArgType::Address => Ok(TransactionArgument::Address(txn_arg_parser(
                &self.arg, "address",
            )?)),
            FunctionArgType::Bool => Ok(TransactionArgument::Bool(txn_arg_parser(
                &self.arg, "bool",
            )?)),
            FunctionArgType::Hex => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg, "hex",
            )?)),
            FunctionArgType::HexArray => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg,
                "hex_array",
            )?)),
            FunctionArgType::String => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg, "string",
            )?)),
            FunctionArgType::U8 => Ok(TransactionArgument::U8(txn_arg_parser(&self.arg, "u8")?)),
            FunctionArgType::U16 => Ok(TransactionArgument::U16(txn_arg_parser(&self.arg, "u16")?)),
            FunctionArgType::U32 => Ok(TransactionArgument::U32(txn_arg_parser(&self.arg, "u32")?)),
            FunctionArgType::U64 => Ok(TransactionArgument::U64(txn_arg_parser(&self.arg, "u64")?)),
            FunctionArgType::U128 => Ok(TransactionArgument::U128(txn_arg_parser(
                &self.arg, "u128",
            )?)),
            FunctionArgType::U256 => Ok(TransactionArgument::U256(txn_arg_parser(
                &self.arg, "u256",
            )?)),
            FunctionArgType::Raw => Ok(TransactionArgument::U8Vector(txn_arg_parser(
                &self.arg, "raw",
            )?)),
            arg_type => Err(CliError::CommandArgumentError(format!(
                "Input type {} not supported",
                arg_type
            ))),
        }
    }
}
