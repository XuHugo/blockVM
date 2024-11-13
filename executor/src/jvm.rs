use jni::{
    objects::{JValue, JObject, JClass, JString},
    sys::{jstring, jobject, jint, jlong},
    strings::{JNIString},
    errors,
    InitArgsBuilder,JavaVM, AttachGuard, JNIVersion, JNIEnv, NativeMethod, signature,
};

use std::{
    ffi::c_void,
    time,
    collections::{BTreeMap, LinkedList},
    env,
    path::{Path, PathBuf},
    process::Command,
    sync::{MutexGuard,Arc, Mutex, Once},
};

use crate::types::{Context, ContractKind, ContractResult, ExecKind, FunName, VMType, ADDRESS_SIZE, Context_JVM, Result_JVM};
use wasm_chain_integration::*;
use concordium_contracts_common::{ Amount, Address, ChainMetadata, Timestamp, AccountAddress, DID};
use anyhow::{ bail, ensure, anyhow, Error, Result};

use lazy_static::lazy_static;



pub fn jvm() -> &'static Arc<JavaVM> {
    static mut JVM: Option<Arc<JavaVM>> = None;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option("-Xcheck:jni")
            .build()
            .unwrap_or_else(|e| panic!("{:#?}", e));

        let jvm = JavaVM::new(jvm_args).unwrap_or_else(|e| panic!("{:#?}", e));

        unsafe {
            JVM = Some(Arc::new(jvm));
        }
    });

    unsafe { JVM.as_ref().unwrap() }
}

pub fn attach_current_thread() -> AttachGuard<'static> {
    jvm()
        .attach_current_thread()
        .expect("failed to attach jvm thread")
}

pub fn attach_current_thread_as_daemon() -> JNIEnv<'static> {
    jvm()
        .attach_current_thread_as_daemon()
        .expect("failed to attach jvm daemon thread")
}

pub fn attach_current_thread_permanently() -> JNIEnv<'static> {
    jvm()
        .attach_current_thread_permanently()
        .expect("failed to attach jvm thread permanently")
}

pub fn detach_current_thread() {
    jvm().detach_current_thread()
}

pub fn print_exception(env: &JNIEnv) {
    let exception_occurred = env.exception_check().unwrap_or_else(|e| panic!("{:?}", e));
    if exception_occurred {
        env.exception_describe()
            .unwrap_or_else(|e| panic!("{:?}", e));
    }
}

pub fn unwrap<T>(env: &JNIEnv, res: errors::Result<T>) -> T {
    res.unwrap_or_else(|e| {
        print_exception(env);
        panic!("{:#?}", e);
    })
}

lazy_static! {
        static ref VM: JavaVM = {
            let args = InitArgsBuilder::new()
                .version(JNIVersion::V8)
        .option("-Xcheck:jni")
                .build()
                .unwrap();
            JavaVM::new(args).unwrap()
        };
    }

static mut CONTEXT: Context_JVM = Context_JVM{
    version:1,
    kind: ExecKind::Init,
    contract_name: String::new(),
    func_name: String::new(),
    param: String::new(),
    sender: String::new(),
    invoker:  String::new(),
    owner:  String::new(),
    logs: String::new(),
    self_balance:  0,
    self_address: String::new(),
    metadata:        ChainMetadata {
        slot_time: Timestamp{milliseconds:0},
        height: 0,
        tx_hash: String::new(),
    },
    returndata: String::new(),
    gas_counter: 0,
    gas_limit:0,
    gas_outof: false,
    gas:false,
    did:DID{},
};


fn get_block_height(env: JNIEnv, object: JObject)-> jlong{
    unsafe{
        CONTEXT.metadata.height as i64
    }
}

fn get_tx_hash(env: JNIEnv, object: JObject)-> jstring{
    let ret = unsafe{
        CONTEXT.metadata.tx_hash.clone()
    };
    env.new_string(ret).unwrap().into_inner()
}

fn get_self_address(env: JNIEnv, object: JObject)-> jstring{
    let ret = unsafe{
        String::from(CONTEXT.self_address.clone())
    };
    env.new_string(ret).unwrap().into_inner()
}
fn get_time(env: JNIEnv, object: JObject)-> jlong{
    let ret = unsafe{
        CONTEXT.metadata.slot_time.timestamp_millis()
    };
    ret as i64
}
fn get_sender(env: JNIEnv, object: JObject)-> jstring{
    let ret = unsafe{
        String::from(CONTEXT.self_address.clone())
    };
    env.new_string(ret).unwrap().into_inner()
}
fn get_invoker(env: JNIEnv, object: JObject)-> jstring{
    let ret = unsafe{
        String::from(CONTEXT.invoker.clone())
    };
    env.new_string(ret).unwrap().into_inner()
}

fn get_owner(env: JNIEnv, object: JObject)-> jstring{
    let ret = unsafe{
        String::from(CONTEXT.owner.clone())
    };
    env.new_string(ret).unwrap().into_inner()
}

fn get_state(env: JNIEnv, object: JObject, key:jstring)-> jstring{
    env.new_string("666").unwrap().into_inner()
}

fn set_state(env: JNIEnv, object: JObject, key:jstring, value:jstring)-> jstring{
    env.new_string("666").unwrap().into_inner()
}

fn get_self_balance(env: JNIEnv, object: JObject)-> jlong{
    unsafe{
        CONTEXT.self_balance as i64
    }
}

fn event(env: JNIEnv, object: JObject, event: jstring){
    let ret =match env.get_string(JString::from(event)){
        Ok(r) => r,
        Err(e) => return,
    };
    match ret.to_str(){
        Ok(r)=> unsafe{
            println!("event:{:?}", r);
            CONTEXT.logs= r.to_string();
            return
        },
        Err(e) => return,
    }
}

fn call(env: JNIEnv, object: JObject, class: jstring, par: jstring)-> jstring{
    // let c = match env.get_string(JString::from(class)) {
    //     Ok(r) => match r.to_str() {
    //         Ok(v) => v.to_string(),
    //         Err(_) => return "".to_string(),
    //     },
    //     Err(_e) => return "".to_string(),
    // };

    // // let s = match env.get_string(JString::from(sig)) {
    // //     Ok(r) => match r.to_str() {
    // //         Ok(v) => v.to_string(),
    // //         Err(_) => return "".to_string(),
    // //     },
    // //     Err(_e) => return "".to_string(),
    // // };

    // let p = match env.get_string(JString::from(par)) {
    //     Ok(r) => match r.to_str() {
    //         Ok(v) => v.to_string(),
    //         Err(_) => return "".to_string(),
    //     },
    //     Err(_e) => return "".to_string(),
    // };
    // let contract_binary = "";
    // let contract_class = match env.find_class(c.clone()){
    //     Ok(c) => c,
    //     Err(e) =>{
    //         env.exception_clear()?;
    //         let contract_class =  env.define_class(c.clone(), JObject::null(), contract_binary)?;
    //         contract_class
    //     }
    // };
    // let contract_obj = env.new_object(contract_class, "()V", &[]).unwrap();

    // let s = env.new_string(p)?;
    // let param = JValue::Object(*s);
    // let val = env.call_method(contract_obj, ctx.func_name, "(Ljava/lang/String;)Ljava/lang/String;", &[param])?;

    // let r_val =match env.get_string(JString::from(val.l()?)){
    //     Ok(r)=>r,
    //     Err(_)=>return Err(errors::Error::JavaException),
    // };
    // let t = match  r_val.to_str(){
    //     Ok(r) => r,
    //     Err(_) => return Err(errors::Error::JavaException),
    // };

    env.new_string("666").unwrap().into_inner()
}

//Result<InitResult>
pub fn init_jvm(geeco_binary: &[u8], contract_binary: &[u8],amount: i64, ctx:Context_JVM) -> anyhow::Result<Result_JVM, errors::Error> {

    let env: AttachGuard = VM.attach_current_thread()?;
    unsafe{
        CONTEXT = ctx.clone();
    }
    let geeco_class = match env.find_class("GeeCo"){
        Ok(c) => c,
        Err(e) =>{
            println!("init geeco{:?}",e);
            env.exception_clear()?;
            let geeco_class = env.define_class("GeeCo", JObject::null(), geeco_binary)?;
            geeco_class
        }
    };
    //let geeco_class = env.define_class("GeeCo", JObject::null(), geeco_binary).unwrap();
    //let geeco_obj = env.new_object(class, "()V", &[]).unwrap();

    let na_fn: Vec<NativeMethod> = vec![
        NativeMethod{name:JNIString::from("get_block_height"), sig:JNIString::from("()J"), fn_ptr:get_block_height as *mut c_void},
        NativeMethod{name:JNIString::from("get_tx_hash"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_tx_hash as *mut c_void},
        NativeMethod{name:JNIString::from("get_self_address"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_self_address as *mut c_void},
        NativeMethod{name:JNIString::from("get_time"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_time as *mut c_void},
        NativeMethod{name:JNIString::from("get_sender"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_sender as *mut c_void},
        NativeMethod{name:JNIString::from("get_invoker"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_invoker as *mut c_void},
        NativeMethod{name:JNIString::from("get_owner"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_owner as *mut c_void},
        NativeMethod{name:JNIString::from("get_state"), sig:JNIString::from("(Ljava/lang/String;)Ljava/lang/String;"), fn_ptr:get_state as *mut c_void},
        NativeMethod{name:JNIString::from("set_state"), sig:JNIString::from("(Ljava/lang/String;Ljava/lang/String;)Z"), fn_ptr:set_state as *mut c_void},
        NativeMethod{name:JNIString::from("get_self_balance"), sig:JNIString::from("()J"), fn_ptr:get_self_balance as *mut c_void},
        NativeMethod{name:JNIString::from("event"), sig:JNIString::from("(Ljava/lang/String;)V"), fn_ptr:event as *mut c_void},
    ];
    env.register_native_methods(geeco_class, &na_fn)?;

    let contract_class = match env.find_class(ctx.contract_name.clone()){
        Ok(c) => c,
        Err(e) =>{
            env.exception_clear()?;
            let contract_class =  env.define_class(ctx.contract_name.clone(), JObject::null(), contract_binary)?;
            contract_class
        }
    };
    let contract_obj = env.new_object(contract_class, "()V", &[])?;

    let s = env.new_string(ctx.param)?;
    let param = JValue::Object(*s);
    let val = env.call_method(contract_obj, "init", "(Ljava/lang/String;)Ljava/lang/String;", &[param])?;

    let r_val =match env.get_string(JString::from(val.l()?)){
        Ok(r)=>r,
        Err(_)=>return Err(errors::Error::JavaException),
    };
    let t = match  r_val.to_str(){
        Ok(r) => r,
        Err(_) => return Err(errors::Error::JavaException),
    };
    //println!("result:{}",r_val.clone());
    Ok(Result_JVM{
        logs: unsafe {
            CONTEXT.logs.clone()
        },
        returndata: t.to_string(),
        gas_counter: 0,
        gas_outof: false,
    })
}

//Result<ReceiveResult>
pub fn call_jvm(geeco_binary: &[u8], contract_binary: &[u8], amount: i64, ctx:Context_JVM) -> anyhow::Result<Result_JVM, errors::Error> {

    let env: AttachGuard = VM.attach_current_thread()?;
    unsafe{
        CONTEXT = ctx.clone();
    }

    let geeco_class = match env.find_class("GeeCo"){
        Ok(c) => c,
        Err(e) =>{
            env.exception_clear()?;
            let geeco_class = env.define_class("GeeCo", JObject::null(), geeco_binary)?;
            geeco_class
        }
    };
    //let geeco_class = env.define_class("GeeCo", JObject::null(), geeco_binary).unwrap();
    //let geeco_obj = env.new_object(class, "()V", &[]).unwrap();

    let na_fn: Vec<NativeMethod> = vec![
        NativeMethod{name:JNIString::from("get_block_height"), sig:JNIString::from("()J"), fn_ptr:get_block_height as *mut c_void},
        NativeMethod{name:JNIString::from("get_tx_hash"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_tx_hash as *mut c_void},
        NativeMethod{name:JNIString::from("get_self_address"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_self_address as *mut c_void},
        NativeMethod{name:JNIString::from("get_time"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_time as *mut c_void},
        NativeMethod{name:JNIString::from("get_sender"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_sender as *mut c_void},
        NativeMethod{name:JNIString::from("get_invoker"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_invoker as *mut c_void},
        NativeMethod{name:JNIString::from("get_owner"), sig:JNIString::from("()Ljava/lang/String;"), fn_ptr:get_owner as *mut c_void},
        NativeMethod{name:JNIString::from("get_state"), sig:JNIString::from("(Ljava/lang/String;)Ljava/lang/String;"), fn_ptr:get_state as *mut c_void},
        NativeMethod{name:JNIString::from("set_state"), sig:JNIString::from("(Ljava/lang/String;Ljava/lang/String;)Z"), fn_ptr:set_state as *mut c_void},
        NativeMethod{name:JNIString::from("get_self_balance"), sig:JNIString::from("()J"), fn_ptr:get_self_balance as *mut c_void},
        NativeMethod{name:JNIString::from("event"), sig:JNIString::from("(Ljava/lang/String;)V"), fn_ptr:event as *mut c_void},
        NativeMethod{name:JNIString::from("call"), sig:JNIString::from("(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"), fn_ptr:call as *mut c_void},
    ];
    env.register_native_methods(geeco_class, &na_fn)?;


    let contract_class = match env.find_class(ctx.contract_name.clone()){
        Ok(c) => c,
        Err(e) =>{
            env.exception_clear()?;
            let contract_class =  env.define_class(ctx.contract_name.clone(), JObject::null(), contract_binary)?;
            contract_class
        }
    };
    let contract_obj = env.new_object(contract_class, "()V", &[]).unwrap();

    let s = env.new_string(ctx.param)?;
    let param = JValue::Object(*s);
    let val = env.call_method(contract_obj, ctx.func_name, "(Ljava/lang/String;)Ljava/lang/String;", &[param])?;

    let r_val =match env.get_string(JString::from(val.l()?)){
        Ok(r)=>r,
        Err(_)=>return Err(errors::Error::JavaException),
    };
    let t = match  r_val.to_str(){
        Ok(r) => r,
        Err(_) => return Err(errors::Error::JavaException),
    };
    //println!("result:{}",r_val.clone());
    Ok(Result_JVM{
        logs: unsafe {
            CONTEXT.logs.clone()
        },
        returndata: t.to_string(),
        gas_counter: 0,
        gas_outof: false,
    })
}
