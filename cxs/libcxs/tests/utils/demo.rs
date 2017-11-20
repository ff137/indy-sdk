extern crate cxs;
extern crate tempfile;
extern crate libc;
extern crate mockito;
extern crate serde_json;

#[macro_use]
use utils::cstring;
use utils::timeout::TimeoutUtils;
use utils::cstring::CStringUtils;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use self::libc::c_char;
use std::thread;
use std::time::Duration;
use std::ffi::CString;
use cxs::api;
use std::sync::Mutex;
use std::sync::mpsc::channel;
lazy_static! {
    static ref COMMAND_HANDLE_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
}
#[allow(unused_assignments)]
#[allow(unused_variables)]
pub extern "C" fn generic_cb(command_handle:u32, err:u32) {
    if err != 0 {panic!("failed connect: {}", err)}
    println!("connection established!");
}
pub fn create_claim_offer(source_id: &str, claim_data_value: serde_json::Value, issuer_did: &str, schema_seq_no: u32) -> (u32, u32){
    let source_id_cstring = CString::new(source_id).unwrap();
    let (sender, receiver) = channel();
    let cb = Box::new(move|err, claim_handle|{sender.send((err, claim_handle)).unwrap();});
    let (command_handle, cb) = closure_to_create_claim(cb);
    let claim_data_str = serde_json::to_string(&claim_data_value).unwrap();
    let claim_data_cstring = CString::new(claim_data_str).unwrap();
    let issuer_did_cstring = CString::new(issuer_did).unwrap();
    let rc = api::issuer_claim::cxs_issuer_create_claim(command_handle,
                                                        source_id_cstring.as_ptr(),
                                                        schema_seq_no,
                                                        issuer_did_cstring.as_ptr(),
                                                        claim_data_cstring.as_ptr(),
                                                        cb);
    assert_eq!(rc, 0);
    receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap()
}

pub fn send_claim_offer(claim_handle: u32, connection_handle: u32) -> u32 {
    let (sender, receiver) = channel();
    let cb = Box::new(move|err|{sender.send(err).unwrap();});
    let (command_handle, cb) = closure_to_send_claim_object(cb);
    let rc = api::issuer_claim::cxs_issuer_send_claim_offer(command_handle,
                                                            claim_handle,
                                                            connection_handle,
                                                            cb);
    assert_eq!(rc,0);
    receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap()
}

pub fn send_claim(claim_handle: u32, connection_handle: u32) -> u32 {
    let (sender, receiver) = channel();
    let cb = Box::new(move|err|{sender.send(err).unwrap();});
    let (command_handle, cb) = closure_to_send_claim_object(cb);
    let rc = api::issuer_claim::cxs_issuer_send_claim(command_handle, claim_handle, connection_handle, cb);
    assert_eq!(rc,0);
    receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap()

}
pub fn deserialize_cxs_object(serialized_connection: &str,f:extern fn(u32, *const c_char, Option<extern fn(u32, u32, u32)>) ->u32 ) -> u32{
    fn closure_to_deserialize_connection(closure: Box<FnMut(u32, u32) + Send>) ->
    (u32,  Option<extern fn( command_handle: u32,
                             err: u32 ,
                             connection_handle: u32)>) {
        lazy_static! { static ref CALLBACK_DESERIALIE_CONNECTION: Mutex<HashMap<u32,
                                        Box<FnMut(u32, u32) + Send>>> = Default::default(); }

        extern "C" fn callback(command_handle: u32, err: u32, connection_handle: u32) {
            let mut callbacks = CALLBACK_DESERIALIE_CONNECTION.lock().unwrap();
            let mut cb = callbacks.remove(&command_handle).unwrap();
            cb(err, connection_handle)
        }

        let mut callbacks = CALLBACK_DESERIALIE_CONNECTION.lock().unwrap();
        let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
        callbacks.insert(command_handle, closure);

        (command_handle, Some(callback))
    }
    let (sender, receiver) = channel();
    let cb = Box::new(move|err, handle|{sender.send((err,handle)).unwrap();});
    let (command_handle, cb) = closure_to_deserialize_connection(cb);
    let rc = f(command_handle,
               CStringUtils::string_to_cstring(String::from(serialized_connection)).as_ptr(),
               cb);
    assert_eq!(rc,0);
    let (err, connection_handle) = receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap();
    assert_eq!(err,0);
    connection_handle

}
pub fn serialize_cxs_object(connection_handle: u32, f:extern fn(u32, u32, Option<extern fn(u32, u32, *const c_char)> ) ->u32) -> u32{
    fn closure_to_serialize_connection(closure: Box<FnMut(u32) + Send>) ->
    (u32, Option<extern fn( command_handle: u32, err: u32 , claim_string: *const c_char)>) {
        lazy_static! { static ref CALLBACKS_SERIALIZE_CONNECTION: Mutex<HashMap<u32,
                                        Box<FnMut(u32) + Send>>> = Default::default(); }

        extern "C" fn callback(command_handle: u32, err: u32, claim_string: *const c_char) {
            let mut callbacks = CALLBACKS_SERIALIZE_CONNECTION.lock().unwrap();
            let mut cb = callbacks.remove(&command_handle).unwrap();
            assert_eq!(err, 0);
            if claim_string.is_null() {
                panic!("claim_string is empty");
            }
            check_useful_c_str!(claim_string, ());
            println!("successfully called serialize_cb: {}", claim_string);
            cb(err)
        }

        let mut callbacks = CALLBACKS_SERIALIZE_CONNECTION.lock().unwrap();
        let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
        callbacks.insert(command_handle, closure);

        (command_handle, Some(callback))
    }
    let (sender, receiver) = channel();
    let cb = Box::new(move |err|{sender.send(err).unwrap();});
    let (command_handle, cb) = closure_to_serialize_connection(cb);
    let rc = f(command_handle,
               connection_handle,
               cb);

    assert_eq!(rc, 0);
    receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap()
}

pub fn wait_for_updated_state(handle: u32, target_state:u32, f: extern fn(u32, u32, Option<extern fn(u32, u32, u32)>)->u32)->u32{
    //  Update State, wait for connection *********************************************
    let mut state = 0;
    while state != target_state {
        let (sender, receiver) = channel();
        let (command_handle, cb) = closure_to_update_state(Box::new(move |state| { sender.send(state).unwrap(); }));
        thread::sleep(Duration::from_millis(5000));
        let err = f(command_handle, handle, cb);
        assert_eq!(err,0);
        state = receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap();
    }
    state
}
pub fn closure_to_create_connection_cb(closure: Box<FnMut(u32, u32) + Send>) ->
(u32,
 Option<extern fn(
     command_handle: u32,
     err: u32,
     connection_handle: u32)>) {
    lazy_static! {
            static ref CALLBACKS_CREATE_CONNECTION: Mutex<HashMap<u32, Box<FnMut(u32, u32) + Send>>> = Default::default();
        }

    extern "C" fn callback(command_handle: u32, err: u32, connection_handle: u32) {
        let mut callbacks = CALLBACKS_CREATE_CONNECTION.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(err, connection_handle)
    }

    let mut callbacks = CALLBACKS_CREATE_CONNECTION.lock().unwrap();
    let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
    callbacks.insert(command_handle, closure);

    (command_handle, Some(callback))
}
pub fn closure_to_connect_cb(closure: Box<FnMut(u32) + Send>) -> (u32,
                                                                  Option<extern fn(
                                                                      command_handle: u32,
                                                                      err: u32 )>) {
    lazy_static! {
        static ref CALLBACKS: Mutex<HashMap<u32, Box<FnMut(u32) + Send>>> = Default::default();
    }
    // this is the only difference between the two closure converters
    extern "C" fn callback(command_handle: u32, err: u32) {
        let mut callbacks = CALLBACKS.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(err)
    }

    let mut callbacks = CALLBACKS.lock().unwrap();
    let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
    callbacks.insert(command_handle, closure);

    (command_handle, Some(callback))
}
pub fn closure_to_update_state(closure: Box<FnMut(u32) + Send>) ->
(u32,
 Option<extern fn(
     command_handle: u32,
     err: u32,
     connection_handle: u32)>) {
    lazy_static! { static ref CALLBACKS_GET_STATE: Mutex<HashMap<u32, Box<FnMut(u32) + Send>>> = Default::default(); }

    #[allow(unused_variables)]
    extern "C" fn callback(command_handle: u32, err: u32, state: u32) {
        let mut callbacks = CALLBACKS_GET_STATE.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(state)
    }

    let mut callbacks = CALLBACKS_GET_STATE.lock().unwrap();
    let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
    callbacks.insert(command_handle, closure);

    (command_handle, Some(callback))
}

pub fn closure_to_create_claim(closure: Box<FnMut(u32, u32) + Send>) ->
(u32, Option<extern fn( command_handle: u32, err: u32, claim_handle: u32)>) {
    lazy_static! { static ref CALLBACKS_CREATE_CLAIM: Mutex<HashMap<u32, Box<FnMut(u32, u32) + Send>>> = Default::default(); }

    extern "C" fn callback(command_handle: u32, err: u32, claim_handle: u32) {
        let mut callbacks = CALLBACKS_CREATE_CLAIM.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(err, claim_handle)
    }

    let mut callbacks = CALLBACKS_CREATE_CLAIM.lock().unwrap();
    let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
    callbacks.insert(command_handle, closure);

    (command_handle, Some(callback))
}
pub fn closure_to_send_claim_object(closure: Box<FnMut(u32) + Send>) -> (u32, Option<extern fn(command_handle: u32, err: u32 )>) {
    lazy_static! { static ref CALLBACKS_SEND_CLAIM: Mutex<HashMap<u32, Box<FnMut(u32) + Send>>> = Default::default(); }

    extern "C" fn callback(command_handle: u32, err: u32) {
        let mut callbacks = CALLBACKS_SEND_CLAIM.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(err)
    }

    let mut callbacks = CALLBACKS_SEND_CLAIM.lock().unwrap();
    let command_handle = (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as u32;
    callbacks.insert(command_handle, closure);

    (command_handle, Some(callback))
}
