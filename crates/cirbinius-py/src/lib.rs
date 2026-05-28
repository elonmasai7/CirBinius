use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use cirbinius_sdk::CirbiniusClient;

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_error(msg: String) {
    if let Ok(mut err) = LAST_ERROR.lock() {
        *err = Some(msg);
    }
}

fn take_error() -> Option<String> {
    if let Ok(mut err) = LAST_ERROR.lock() {
        err.take()
    } else {
        None
    }
}

struct PyClient {
    client: CirbiniusClient,
    runtime: tokio::runtime::Runtime,
}

unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null pointer".into());
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|e| format!("invalid UTF-8: {e}"))
}

fn cstr_to_option(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

unsafe fn string_to_c(s: String) -> *mut c_char {
    CString::new(s)
        .unwrap_or_default()
        .into_raw()
}

unsafe fn client_ref<'a>(ptr: *mut std::os::raw::c_void) -> Option<&'a mut PyClient> {
    if ptr.is_null() {
        set_error("null client pointer".into());
        return None;
    }
    unsafe { (ptr as *mut PyClient).as_mut() }
}

#[unsafe(no_mangle)]
pub extern "C" fn cirbinius_client_new(
    host: *const c_char,
    port: u16,
    api_key: *const c_char,
) -> *mut std::os::raw::c_void {
    let host_str = match unsafe { cstr_to_str(host) } {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            set_error(format!("failed to create tokio runtime: {e}"));
            return std::ptr::null_mut();
        }
    };

    let mut client = CirbiniusClient::new(host_str.to_string(), port);

    if let Some(key) = cstr_to_option(api_key) {
        client = client.with_api_key(key);
    }

    let py_client = Box::new(PyClient { client, runtime });
    Box::into_raw(py_client) as *mut std::os::raw::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn cirbinius_client_free(client: *mut std::os::raw::c_void) {
    if client.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(client as *mut PyClient));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cirbinius_request(
    client: *mut std::os::raw::c_void,
    method: *const c_char,
    path: *const c_char,
    body: *const c_char,
) -> *mut c_char {
    let pyc = match unsafe { client_ref(client) } {
        Some(c) => c,
        None => return std::ptr::null_mut(),
    };

    let method_str = match unsafe { cstr_to_str(method) } {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    let path_str = match unsafe { cstr_to_str(path) } {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return std::ptr::null_mut();
        }
    };

    let body_val: Option<serde_json::Value> = cstr_to_option(body)
        .and_then(|s| serde_json::from_str(&s).ok());

    let result = pyc.runtime.block_on(async {
        let result_json: serde_json::Value = pyc.client
            .request_typed(method_str, path_str, body_val.as_ref())
            .await?;
        Ok::<_, cirbinius_sdk::SdkError>(result_json)
    });

    match result {
        Ok(val) => unsafe { string_to_c(serde_json::to_string(&val).unwrap_or_default()) },
        Err(e) => {
            set_error(e.to_string());
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cirbinius_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(s));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cirbinius_get_last_error() -> *mut c_char {
    match take_error() {
        Some(msg) => unsafe { string_to_c(msg) },
        None => std::ptr::null_mut(),
    }
}
