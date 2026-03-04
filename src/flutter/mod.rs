//! Flutter FFI interface for Seikaku Engine.
//!
//! Exposes C-compatible functions that can be called from Dart via `dart:ffi`.
//!
//! ## Usage flow
//! 1. `seikaku_init(path)` – load pb2 data files from the given directory, returns an opaque engine handle.
//! 2. `seikaku_calculate(engine, fit_json, skills_json)` – calculate ship fit statistics, returns JSON.
//! 3. `seikaku_load_eft(engine, eft_text)` – parse EFT format text, returns JSON of parsed fit.
//! 4. `seikaku_free_string(ptr)` – free a string returned by the engine.
//! 5. `seikaku_free(engine)` – free the engine handle.

use std::collections::BTreeMap;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;

use crate::calculate;
use crate::data_types;
use crate::rust::{Data, InfoMain, InfoNameMain};

/// Opaque engine handle that holds all loaded game data.
pub struct Engine {
    data: Data,
}

/// Initialize the engine by loading pb2 data files from the given directory.
///
/// # Parameters
/// - `path`: Null-terminated UTF-8 string – the directory containing `*.pb2` files
///   (dogmaAttributes.pb2, dogmaEffects.pb2, typeDogma.pb2, types.pb2).
///
/// # Returns
/// An opaque pointer to the engine, or null on failure.
#[no_mangle]
pub extern "C" fn seikaku_init(path: *const c_char) -> *mut Engine {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let path_buf = PathBuf::from(path_str);
    if !path_buf.exists() {
        return std::ptr::null_mut();
    }

    let data = Data::new(&path_buf);
    Box::into_raw(Box::new(Engine { data }))
}

/// Calculate ship fit statistics.
///
/// # Parameters
/// - `engine`: Engine handle obtained from `seikaku_init`.
/// - `fit_json`: Null-terminated UTF-8 JSON string of `EsfFit`.
///   Example:
///   ```json
///   {
///     "ship_type_id": 24690,
///     "modules": [
///       {
///         "type_id": 2048,
///         "slot": { "type": "High", "index": 0 },
///         "state": "Active",
///         "charge": null
///       }
///     ],
///     "drones": []
///   }
///   ```
/// - `skills_json`: Null-terminated UTF-8 JSON string of skills map.
///   Example: `{"3300": 5, "3301": 4}`
///
/// # Returns
/// A newly allocated null-terminated UTF-8 JSON string with the calculation result.
/// Caller must free with `seikaku_free_string`. Returns null on failure.
#[no_mangle]
pub extern "C" fn seikaku_calculate(
    engine: *mut Engine,
    fit_json: *const c_char,
    skills_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() || fit_json.is_null() || skills_json.is_null() {
        return std::ptr::null_mut();
    }

    let engine = unsafe { &*engine };

    let fit_str = match unsafe { CStr::from_ptr(fit_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("Invalid UTF-8 in fit_json"),
    };

    let skills_str = match unsafe { CStr::from_ptr(skills_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("Invalid UTF-8 in skills_json"),
    };

    let fit: data_types::EsfFit = match serde_json::from_str(fit_str) {
        Ok(f) => f,
        Err(e) => return make_error(&format!("Failed to parse fit JSON: {}", e)),
    };

    let skills: BTreeMap<String, i32> = match serde_json::from_str(skills_str) {
        Ok(s) => s,
        Err(e) => return make_error(&format!("Failed to parse skills JSON: {}", e)),
    };

    let skills: BTreeMap<i32, i32> = skills
        .into_iter()
        .filter_map(|(k, v)| k.parse::<i32>().ok().map(|k| (k, v)))
        .collect();

    let info = InfoMain::new(fit, skills, &engine.data);
    let statistics = calculate::calculate(&info);

    match serde_json::to_string(&statistics) {
        Ok(json) => string_to_c(json),
        Err(e) => make_error(&format!("Failed to serialize result: {}", e)),
    }
}

/// Parse an EFT (EVE Fitting Tool) format string and return the parsed fit as JSON.
///
/// # Parameters
/// - `engine`: Engine handle obtained from `seikaku_init`.
/// - `eft_text`: Null-terminated UTF-8 EFT text.
///
/// # Returns
/// A newly allocated null-terminated UTF-8 JSON string of `EsfFit`.
/// Caller must free with `seikaku_free_string`. Returns null on failure.
#[cfg(feature = "eft")]
#[no_mangle]
pub extern "C" fn seikaku_load_eft(
    engine: *mut Engine,
    eft_text: *const c_char,
) -> *mut c_char {
    if engine.is_null() || eft_text.is_null() {
        return std::ptr::null_mut();
    }

    let engine = unsafe { &*engine };

    let eft_str = match unsafe { CStr::from_ptr(eft_text) }.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("Invalid UTF-8 in eft_text"),
    };

    let info_name = InfoNameMain::new(&engine.data);
    let eft_string = eft_str.to_string();

    match crate::eft::load_eft(&info_name, &eft_string) {
        Ok(eft_fit) => match serde_json::to_string(&eft_fit.esf_fit) {
            Ok(json) => string_to_c(json),
            Err(e) => make_error(&format!("Failed to serialize EFT fit: {}", e)),
        },
        Err(e) => make_error(&format!("Failed to parse EFT: {}", e)),
    }
}

/// Free the engine handle.
///
/// # Safety
/// The pointer must have been obtained from `seikaku_init` and must not be used after this call.
#[no_mangle]
pub extern "C" fn seikaku_free(engine: *mut Engine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Free a string that was returned by the engine (e.g. from `seikaku_calculate`).
///
/// # Safety
/// The pointer must have been obtained from an engine function and must not be used after this call.
#[no_mangle]
pub extern "C" fn seikaku_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn string_to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn make_error(msg: &str) -> *mut c_char {
    let escaped = msg.replace('\\', "\\\\").replace('"', "\\\"");
    let json = format!(r#"{{"error":"{}"}}"#, escaped);
    string_to_c(json)
}
