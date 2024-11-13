

use crate::{types::{Context}, Contract, VM};
use crate::types::{ContractKind, ContractResult, ContractError, FunName, ExecKind, VMKind};
use crate::wasmtime::{receive_wasm as receive_wasm_wasmtime, init_wasm as init_wasm_wasmtime, receive_wasm_aot as receive_wasm_wasmtime_aot,};
use crate::geecowasm::{receive_wasm as receive_wasm_geeco, init_wasm as init_wasm_geeco};
use wasm_chain_integration::{*, ReceiveResult, Action, InitResult};
use storage::key_value_db::KeyValueDB;
use concordium_contracts_common::{*, schema};
use crate::gas::{WasmCosts, gas_rules};
use parity_wasm::{
    elements::{self, Deserialize},
    peek_size,
};
use pwasm_utils::{self, rules};

pub struct Executor<'a> {
    pub db : KeyValueDB,
    pub context : Context<'a>,
    pub contractkind: ContractKind,
    pub vm_kind:VMKind,
}

impl<'a> Contract for Executor<'a>{
    fn exec(&mut self, binary:Option<&[u8]>, amount:i64) -> Result<ContractResult, ContractError>{

        match self.context.kind{
            ExecKind::Init => {
                self.init_wasm(binary.unwrap(),amount)
            },
            ExecKind::Call => {
                self.call_wasm(amount)
            },
        }
    }

}

impl<'a>  Executor<'a> {

    pub fn call_wasm(&mut self, amount: i64) -> Result<ContractResult, ContractError> {
        let mut func_name = String::new();
        if let FunName::CallName(c, f) = self.context.func_name.clone() {
            func_name = format!("{}.{}", c, f);
        } else {
            return Err(ContractError::Other("call func name error!".to_string()))
        }

        //get code and state from db
        let mut binary: Vec<u8> = Vec::new();
        self.db.lock().get_bytes(&self.context.self_address.0, &mut binary);
        let state_addr = msp::HashInstanceRef.read().hash(&self.context.self_address.0,);
        let mut state:Vec<u8> = Vec::new();
        self.db.lock().get_bytes(&state_addr, &mut state);
        println!("recv:{:?}", state.clone());
        self.context.state.state = state;



        let ret = match self.vm_kind{
            VMKind::WasmTime =>{
                let ret = match receive_wasm_wasmtime(&func_name, self.context.clone(), &binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        println!("receive_wasm=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::WasmTimeAOT =>{
                let ret = match receive_wasm_wasmtime_aot(&func_name, self.context.clone(), &binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        println!("receive_wasm=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::GeeCo =>{
                let ret = match receive_wasm_geeco(&func_name, self.context.clone(), &binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        println!("geeco_receive_wasm=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
        };
        // let ret = match receive_wasm(&func_name, self.context.clone(), &binary, amount) //.expect("receive_wasm failed.");
        // {
        //     Ok(ret) => ret,
        //     Err(e) => {
        //         println!("receive_wasm=>{:?}", e);
        //         return Err(ContractError::Other(e.to_string()))
        //     },
        // };
        match ret {
            ReceiveResult::Success {
                logs,
                state,
                actions,
                returndata,
                remaining_energy,
            } => {
                println!("logs::{:?}", logs);
                println!("actions::{:?}", actions);
                if !returndata.returndata.is_empty(){

                    return Ok(ContractResult::Data {
                        data:returndata,
                        remaining_energy,
                        event:logs,
                    })
                }
                let mut ret:Vec<bool> = Vec::new();
                for (i, action) in actions.iter().enumerate() {

                    match action {
                        Action::Send {
                            data,
                        } => {
                            let name_str = std::str::from_utf8(&data.name)
                                .expect("Target name is not a valid UTF8 sequence.");
                            //transfer & contract :xq
                            // eprintln!(
                            //     "{}: send a message to contract at ({:?}), calling \
                            //                  method {} with amount {} and parameter{:?}",
                            //     i,
                            //     data.to_addr,
                            //     name_str,
                            //     data.amount,
                            //     data.parameter,
                            // );
                            ret.push(true);
                        }
                        Action::SimpleTransfer {
                            data,
                        } => {
                            // eprintln!(
                            //     "{}: simple transfer to account {:?} of amount {}",
                            //     i,
                            //     data.to_addr,
                            //     data.amount
                            // );

                            ret.push(false);
                        }
                        Action::And {
                            l,
                            r,
                        } => {
                            //eprintln!("{}: AND composition of {} and {}", i, l, r);
                            let and = ret[*l as usize] && ret[*r as usize];
                            ret.push(and);
                        },
                        Action::Or {
                            l,
                            r,
                        } => {
                            //eprintln!("{}: OR composition of {} and {}", i, l, r);
                            let or = ret[*l as usize] || ret[*r as usize];
                            ret.push(or);
                        },
                        Action::Accept => {
                            //eprintln!("{}: Accept", i );

                            ret.push(true);
                        },
                        Action::Get => {
                            println!("{:?}: Get", returndata);
                            return Ok(ContractResult::Success {
                                actions : actions.clone(),
                                remaining_energy,
                                event: logs,
                            })
                        },
                    }

                }

                //println!("ret:{:?}",ret);

                self.db.lock().put_bytes(&state_addr, &state.state);

                Ok(ContractResult::Success {
                    actions : actions.clone(),
                    remaining_energy,
                    event:logs,
                })
            }
            ReceiveResult::Reject {
                remaining_energy,
                reason,
            } => {
                eprintln!("Receive call rejected with reason {}", reason);
                Ok(ContractResult::Reject {
                    reason,
                    remaining_energy,
                })
            }
            ReceiveResult::OutOfEnergy => {
                eprintln!("Receive call terminated with: out of energy.");
                Ok(ContractResult::OutOfEnergy)
            }
        }
    }

    pub fn init_wasm(&mut self, binary: &[u8], amount: i64) -> Result<ContractResult, ContractError> {
        let state_addr = msp::HashInstanceRef.read().hash(&self.context.self_address.0);
        let contract_addr = self.context.self_address.0.clone();
        let mut name = String::new();
        if let FunName::InitName(n) = self.context.func_name.clone() {
            name = n;
        } else {
            return Err(ContractError::Other("init func name error!".to_string()))
        }
        let func_name = format!("init_{}", name);
        let ret = match self.vm_kind{
            VMKind::WasmTime | VMKind::WasmTimeAOT =>{
                let ret = match init_wasm_wasmtime(&func_name, self.context.clone(), binary, amount){
                    Ok(ret) => ret,
                    Err(e) => {
                        eprintln!("receive_init=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::GeeCo =>{
                let ret = match init_wasm_geeco(&func_name, self.context.clone(), binary, amount){
                    Ok(ret) => ret,
                    Err(e) => {
                        eprintln!("receive_init=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
        };
        // let ret = match init_wasm(&func_name, self.context.clone(), binary, amount){
        //     Ok(ret) => ret,
        //     Err(e) => {
        //         eprintln!("receive_init=>{:?}", e);
        //         return Err(ContractError::Other(e.to_string()))
        //     },
        // };
        match ret {
            InitResult::Success {
                logs,
                state,
                remaining_energy,
            } => {

                self.db.lock().put_bytes(&contract_addr, &binary.to_vec());
                self.db.lock().put_bytes(&state_addr, &state.state);
                Ok(ContractResult::Success {
                    remaining_energy,
                    actions: vec![],
                    event: logs,
                })
            }
            InitResult::Reject {
                remaining_energy,
                reason,
            } => {
                eprintln!("Init call rejected with reason {}.", reason);
                Ok(ContractResult::Reject {
                    reason,
                    remaining_energy,
                })
            }
            InitResult::OutOfEnergy => {
                eprintln!("Init call terminated with out of energy.");
                Ok(ContractResult::OutOfEnergy)
            }
        }
    }

    pub fn preprocessing(&mut self, binary:&[u8]) -> Result<(), ContractError>{
        let schema = match generate_contract_schema(binary) {
            Ok(s) => s,
            Err(e) => return Err(ContractError::Other(e.to_string()))
        };
        println!("@@@@{:?}",schema);
        let schema_opt = Some(schema);
        return if let Some(module_schema) = &schema_opt {
            // View the size of the data
             for (contract_name, contract_schema) in module_schema.contracts.iter() {
                 print_contract_schema(&contract_name, &contract_schema);
             }
            let module_schema_bytes = to_bytes(module_schema);

            let contract_addr = self.context.self_address.0.clone();
            let mut schema_addr = contract_addr.to_vec();
            schema_addr.insert(42, 66);
            self.db.lock().put_bytes(&schema_addr, &module_schema_bytes.to_vec());
            Ok(())
        } else {
            println!("preprocessing error!");
            Err(ContractError::Other("preprocessing can't find module.".to_string()))
        }
    }

    pub fn preprocessing_gas(&mut self, binary:&[u8]) -> Result<Vec<u8>, ContractError>{
        let deserialized_module = elements::Module::from_bytes(binary)
            .map_err(|err| ContractError::Other(format!("Error deserializing contract code ({:?})", err)));
        let dm = match deserialized_module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: can't deserialized module".to_string()))
        };
        let module =
            pwasm_utils::inject_gas_counter(dm, &gas_rules(&WasmCosts::default()), "gas")
                .map_err(|_| ContractError::Other(format!("Wasm contract error: bytecode invalid")));
        let module = match module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: can't ingect gas counter".to_string()))
        };
        let module = module.to_bytes();

        let module = match module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: module ".to_string()))
        };

        return Ok(module)
    }
}

pub fn preprocessing(binary:&[u8], address:&str, db:&KeyValueDB) -> Result<(), ContractError>{
    let schema = match generate_contract_schema(binary) {
        Ok(s) => s,
        Err(e) => return Err(ContractError::Other(e.to_string()))
    };
    //println!("@@@@{:?}",schema);
    let schema_opt = Some(schema);
    return if let Some(module_schema) = &schema_opt {
        // View the size of the data
         for (contract_name, contract_schema) in module_schema.contracts.iter() {
             print_contract_schema(&contract_name, &contract_schema);
         }
        let module_schema_bytes = to_bytes(module_schema);

        let schema_addr0 = msp::HashInstanceRef.read().hash(address.as_bytes());
        let schema_addr = msp::HashInstanceRef.read().hash(&schema_addr0);
        println!("schema:{:?}",schema_addr);
        // let contract_addr = address.0.clone();
        // let mut schema_addr = contract_addr.to_vec();
        // schema_addr.insert(42, 66);
        db.lock().put_bytes(&schema_addr, &module_schema_bytes.to_vec());
        Ok(())
    } else {
        println!("preprocessing error!");
        Err(ContractError::Other("preprocessing can't find module.".to_string()))
    }
}

// print schema infomation
fn print_contract_schema(contract_name: &str, contract_schema: &concordium_contracts_common::schema::Contract, ) {
    let max_length_receive_opt =
        contract_schema.receive.iter().map(|(n, _)| n.chars().count()).max();
    let colon_position = max_length_receive_opt.map(|m| m.max(5)).unwrap_or(5);
    println!(
        "\n     Contract schema: '{}' in total {} B.",
        contract_name,
        to_bytes(contract_schema).len()
    );
    if let Some(state_schema) = &contract_schema.state {
        println!("       state   : {} B", to_bytes(state_schema).len());
    }
    if let Some(event_schema) = &contract_schema.event {
        println!("       event   : {} B", to_bytes(event_schema).len());
    }
    if let Some(init_schema) = &contract_schema.init {
        println!("       init    : {} B", to_bytes(init_schema).len())
    }

    if !contract_schema.receive.is_empty() {
        println!("       receive");
        for (method_name, param_type) in contract_schema.receive.iter() {
            println!(
                "        - {:width$} : {} B",
                format!("'{}'", method_name),
                to_bytes(param_type).len(),
                width = colon_position + 2
            );
        }
    }
}

pub struct Runtime<'a>{
    pub context: Context<'a>,
    pub vm_kind: VMKind,
}

impl<'a> VM for Runtime<'a>{
    fn run(&self, code:&[u8], amount:i64) -> Result<ContractResult, ContractError>{

        match self.context.kind{
            ExecKind::Init => {
                self.init_wasm(code, amount)
            },
            ExecKind::Call => {
                self.call_wasm(code, amount)
            },
        }
    }

}

impl <'a> Runtime<'a>{
    pub fn call_wasm(&self, binary: &[u8], amount: i64) -> Result<ContractResult, ContractError> {
        let mut func_name = String::new();
        if let FunName::CallName(c, f) = self.context.func_name.clone() {
            func_name = format!("{}.{}", c, f);
        } else {
            return Err(ContractError::Other("call func name error!".to_string()))
        }

        let ret = match self.vm_kind{
            VMKind::WasmTime =>{
                let ret = match receive_wasm_wasmtime(&func_name, self.context.clone(), binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::WasmTimeAOT =>{
                let ret = match receive_wasm_wasmtime_aot(&func_name, self.context.clone(), binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::GeeCo =>{
                let ret = match receive_wasm_geeco(&func_name, self.context.clone(), binary, amount) //.expect("receive_wasm failed.");
                {
                    Ok(ret) => ret,
                    Err(e) => {
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
        };

        match ret {
            ReceiveResult::Success {
                logs,
                state,
                actions,
                returndata,
                remaining_energy,
            } => {
                //println!("logs::{:?}", logs);
                //println!("actions::{:?}", actions);
                if !returndata.returndata.is_empty(){

                    return Ok(ContractResult::Data {
                        data:returndata,
                        remaining_energy,
                        event:logs,
                    })
                }
                let mut ret:Vec<bool> = Vec::new();
                for (i, action) in actions.iter().enumerate() {

                    match action {
                        Action::Send {
                            data,
                        } => {
                            let name_str = std::str::from_utf8(&data.name)
                                .expect("Target name is not a valid UTF8 sequence.");
                            //transfer & contract :xq
                            // eprintln!(
                            //     "{}: send a message to contract at ({:?}), calling \
                            //                  method {} with amount {} and parameter{:?}",
                            //     i,
                            //     data.to_addr,
                            //     name_str,
                            //     data.amount,
                            //     data.parameter,
                            // );
                            ret.push(true);
                        }
                        Action::SimpleTransfer {
                            data,
                        } => {
                            // eprintln!(
                            //     "{}: simple transfer to account {:?} of amount {}",
                            //     i,
                            //     data.to_addr,
                            //     data.amount
                            // );

                            ret.push(false);
                        }
                        Action::And {
                            l,
                            r,
                        } => {
                            //eprintln!("{}: AND composition of {} and {}", i, l, r);
                            let and = ret[*l as usize] && ret[*r as usize];
                            ret.push(and);
                        },
                        Action::Or {
                            l,
                            r,
                        } => {
                            //eprintln!("{}: OR composition of {} and {}", i, l, r);
                            let or = ret[*l as usize] || ret[*r as usize];
                            ret.push(or);
                        },
                        Action::Accept => {
                            //eprintln!("{}: Accept", i );

                            ret.push(true);
                        },
                        Action::Get => {
                            println!("{:?}: Get", returndata);
                            return Ok(ContractResult::Success {
                                actions : actions.clone(),
                                remaining_energy,
                                event: logs,
                            })
                        },
                    }

                }

                //println!("ret:{:?}",ret);

                //self.db.lock().put_bytes(&state_addr, &state.state);

                Ok(ContractResult::Success {
                    actions : actions.clone(),
                    remaining_energy,
                    event:logs,
                })
            }
            ReceiveResult::Reject {
                remaining_energy,
                reason,
            } => {
                eprintln!("Receive call rejected with reason {}", reason);
                Ok(ContractResult::Reject {
                    reason,
                    remaining_energy,
                })
            }
            ReceiveResult::OutOfEnergy => {
                eprintln!("Receive call terminated with: out of energy.");
                Ok(ContractResult::OutOfEnergy)
            }
        }
    }

    pub fn init_wasm(&self, binary: &[u8], amount: i64) -> Result<ContractResult, ContractError> {
        //let state_addr = msp::HashInstanceRef.read().hash(&self.context.self_address.0);
        let contract_addr = self.context.self_address.0.clone();
        let mut name = String::new();
        if let FunName::InitName(n) = self.context.func_name.clone() {
            name = n;
        } else {
            return Err(ContractError::Other("init func name error!".to_string()))
        }
        let func_name = format!("init_{}", name);
        let ret = match self.vm_kind{
            VMKind::WasmTime =>{
                let ret = match init_wasm_wasmtime(&func_name, self.context.clone(), binary, amount){
                    Ok(ret) => ret,
                    Err(e) => {
                        //eprintln!("receive_init=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::WasmTimeAOT =>{
                let ret = match init_wasm_wasmtime(&func_name, self.context.clone(), binary, amount){
                    Ok(ret) => ret,
                    Err(e) => {
                        //eprintln!("receive_init=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
            VMKind::GeeCo =>{
                let ret = match init_wasm_geeco(&func_name, self.context.clone(), binary, amount){
                    Ok(ret) => ret,
                    Err(e) => {
                        //eprintln!("receive_init=>{:?}", e);
                        return Err(ContractError::Other(e.to_string()))
                    },
                };
                ret
            },
        };
        // let ret = match init_wasm(&func_name, self.context.clone(), binary, amount){
        //     Ok(ret) => ret,
        //     Err(e) => {
        //         eprintln!("receive_init=>{:?}", e);
        //         return Err(ContractError::Other(e.to_string()))
        //     },
        // };
        match ret {
            InitResult::Success {
                logs,
                state,
                remaining_energy,
            } => {

                //self.db.lock().put_bytes(&contract_addr, &binary.to_vec());
                //self.db.lock().put_bytes(&state_addr, &state.state);
                Ok(ContractResult::Success {
                    remaining_energy,
                    actions: vec![],
                    event: logs,
                })
            }
            InitResult::Reject {
                remaining_energy,
                reason,
            } => {
                eprintln!("Init call rejected with reason {}.", reason);
                Ok(ContractResult::Reject {
                    reason,
                    remaining_energy,
                })
            }
            InitResult::OutOfEnergy => {
                eprintln!("Init call terminated with out of energy.");
                Ok(ContractResult::OutOfEnergy)
            }
        }
    }

    pub fn preprocessing(&mut self, binary:&[u8]) -> Result<(), ContractError>{
        let schema = match generate_contract_schema(binary) {
            Ok(s) => s,
            Err(e) => return Err(ContractError::Other(e.to_string()))
        };
        println!("@@@@{:?}",schema);
        let schema_opt = Some(schema);
        return if let Some(module_schema) = &schema_opt {
            // View the size of the data
             for (contract_name, contract_schema) in module_schema.contracts.iter() {
                 print_contract_schema(&contract_name, &contract_schema);
             }
            let module_schema_bytes = to_bytes(module_schema);

            let contract_addr = self.context.self_address.0.clone();
            let mut schema_addr = contract_addr.to_vec();
            schema_addr.insert(42, 66);
            //self.db.lock().put_bytes(&schema_addr, &module_schema_bytes.to_vec());
            Ok(())
        } else {
            println!("preprocessing error!");
            Err(ContractError::Other("preprocessing can't find module.".to_string()))
        }
    }

    pub fn preprocessing_gas(&mut self, binary:&[u8]) -> Result<Vec<u8>, ContractError>{
        let deserialized_module = elements::Module::from_bytes(binary)
            .map_err(|err| ContractError::Other(format!("Error deserializing contract code ({:?})", err)));
        let dm = match deserialized_module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: can't deserialized module".to_string()))
        };
        let module =
            pwasm_utils::inject_gas_counter(dm, &gas_rules(&WasmCosts::default()), "gas")
                .map_err(|_| ContractError::Other(format!("Wasm contract error: bytecode invalid")));
        let module = match module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: can't ingect gas counter".to_string()))
        };
        let module = module.to_bytes();

        let module = match module{
            Ok(d) => d,
            _ => return Err(ContractError::Other("gas: module ".to_string()))
        };

        return Ok(module)
    }
}