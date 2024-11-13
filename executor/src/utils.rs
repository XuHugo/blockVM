
use crate::schema_json;
use crate::{types::{ExecKind, ContractError}};
use anyhow::{Result, anyhow, bail, ensure, Context, Error};
use std::{
    fs,
    path::PathBuf,
};
use wasm_chain_integration::*;
use concordium_contracts_common::{*, schema};

//Parse the parameters from json to bytes
pub fn from_json_contract(schema:&[u8], jdata:&[u8], contract_name:String, kind:ExecKind, funcname:String) -> Result<Vec<u8>>{
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file."))?;
    //println!("schema:{:?}", schema);
    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));

    let contract_schema_func_opt =
        contract_schema_opt.and_then(|contract_schema| match kind {
            ExecKind::Init  => contract_schema.init.as_ref(),
            ExecKind::Call  => contract_schema.receive.get(&funcname),
        });
    //println!("contract_schema_func_opt:{:?}", contract_schema_func_opt);
    if contract_schema_func_opt.is_none(){
        //return Err(Error::msg("can not find function in schema"));
        return Ok(vec![])
    }
    let schema_func = contract_schema_func_opt
        .as_ref()
        .context("A schema for the func must be present to use JSON.")?;

    let func_json: serde_json::Value = serde_json::from_slice(&jdata)
        .context("Could not parse func JSON.")?;
    let mut func_bytes = Vec::new();
    schema_json::write_bytes_from_json_schema_type(
        &schema_func,
        &func_json,
        &mut func_bytes,
    ).context("Could not generate func bytes using schema and JSON.")?;
    Ok(func_bytes)

}

pub fn to_json_contract(schema:&[u8], param:&[u8], contract_name:String, kind:ExecKind,funcname:String) ->Result<String>{
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file.")).unwrap();

    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));

    let contract_schema_func_opt =
        contract_schema_opt.and_then(|contract_schema| match kind {
            ExecKind::Init => contract_schema.init.as_ref(),
            ExecKind::Call  => contract_schema.receive.get(&funcname),
            //_ =>  contract_schema.init.as_ref(),
        });
    let schema_func = contract_schema_func_opt.as_ref().context(
        "Schema is required for outputting func in JSON. No schema found the \
                         state in this contract.",
    ).unwrap();

    let json_string = schema_func
        .to_json_string_pretty(&param)
        .map_err(|_| anyhow::anyhow!("Could not output contract func in JSON.")).unwrap();
    Ok(json_string)
    //println!("json func result:{:?}", json_string);
    //fs::write("./output_func.json", json_string).context("Could not write out the func.").unwrap();
}

pub fn from_json_state(schema:&[u8], jdata:&[u8], contract_name:String){
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file.")).unwrap();

    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));

    let contract_schema_state_opt =
        contract_schema_opt.and_then(|contract_schema| contract_schema.state.clone());

    let schema_state = contract_schema_state_opt
        .as_ref()
        .context("A schema for the state must be present to use JSON.").unwrap();
    let state_json: serde_json::Value = serde_json::from_slice(&jdata)
        .context("Could not parse state JSON.").unwrap();
    let mut state_bytes = Vec::new();
    schema_json::write_bytes_from_json_schema_type(
        &schema_state,
        &state_json,
        &mut state_bytes,
    ).context("Could not generate state bytes using schema and JSON.").unwrap();
}

pub fn to_json_state(schema:&[u8],  state:&[u8],  contract_name:String){
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file.")).unwrap();

    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));

    let contract_schema_state_opt =
        contract_schema_opt.and_then(|contract_schema| contract_schema.state.clone());

    let schema_state = contract_schema_state_opt.as_ref().context(
        "Schema is required for outputting state in JSON. No schema found the \
                         state in this contract.",
    ).unwrap();

    let json_string = schema_state
        .to_json_string_pretty(&state)
        .map_err(|_| anyhow::anyhow!("Could not output contract state in JSON.")).unwrap();
    println!("json result:{:?}", json_string);
    fs::write("./output.json", json_string).context("Could not write out the state.").unwrap();
}
//xq log event
pub fn to_json_event(schema:&[u8],  events: Logs,  contract_name:String) -> Result<Vec<String>> {
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file."))?;

    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));

    let contract_schema_event_opt =
        contract_schema_opt.and_then(|contract_schema| contract_schema.event.clone());

    let schema_event = match contract_schema_event_opt.as_ref().context(
        "Schema is required for outputting event in JSON. No schema found the event in this contract.",
    ){
        Ok(v) => v,
        Err(e) => return Ok(vec![])
    };

    let mut event_json = Vec::new();
    for event  in events.logs{
        let json_string = match schema_event
            .to_json_string_pretty(&event)
            .map_err(|_| anyhow::anyhow!("Could not output contract event in JSON."))
            {
                Ok(v) => v,
                Err(e) => String::new(),
            };

        event_json.push(json_string);
    }
    Ok(event_json)
}

pub fn to_json_result(schema:&[u8], param:&[u8], contract_name:String, funcname:String)  -> Result<String> {
    let schema:schema::Module = from_bytes(schema)
        .map_err(|_| anyhow::anyhow!("Could not deserialize schema file."))?;

    let module_schema_opt= Some(schema);
    let contract_schema_opt = module_schema_opt
        .as_ref()
        .and_then(|module_schema| module_schema.contracts.get(&contract_name));
    let mut name = funcname.clone();
    name.insert_str(0, "result_");
    let contract_schema_func_opt =
        contract_schema_opt.and_then(|contract_schema|
            contract_schema.receive.get(&name)
        );

    let schema_func = contract_schema_func_opt.as_ref().context(
        "Schema is required for outputting func in JSON. No schema found the \
                         state in this contract.",
    )?;

    let json_string = schema_func
        .to_json_string_pretty(&param)
        .map_err(|_| anyhow::anyhow!("Could not output contract func in JSON."))?;
    //println!("json func result:{:?}", json_string.clone());
    Ok(json_string)
}
