use std::str::FromStr;

use move_core_types::{
    identifier::Identifier,
    language_storage::ModuleId,
};
use crate::common_ext::types::{CliError, CliTypedResult, load_account_arg};

/// Identifier of a module member (function or struct).
#[derive(Debug, Clone)]
pub struct MemberId {
    pub module_id: ModuleId,
    pub member_id: Identifier,
}

fn parse_member_id(function_id: &str) -> CliTypedResult<MemberId> {
    let ids: Vec<&str> = function_id.split_terminator("::").collect();
    if ids.len() != 3 {
        return Err(CliError::CommandArgumentError(
            "FunctionId is not well formed.  Must be of the form <address>::<module>::<function>"
                .to_string(),
        ));
    }
    let address = load_account_arg(ids.first().unwrap())?;
    let module = Identifier::from_str(ids.get(1).unwrap())
        .map_err(|err| CliError::UnableToParse("Module Name", err.to_string()))?;
    let member_id = Identifier::from_str(ids.get(2).unwrap())
        .map_err(|err| CliError::UnableToParse("Member Name", err.to_string()))?;
    let module_id = ModuleId::new(address, module);
    Ok(MemberId {
        module_id,
        member_id,
    })
}

impl FromStr for MemberId {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_member_id(s)
    }
}