//! C FFI exports for cross-language calling.
//!
//! # Example usage (C/C++):
//! ```c
//! void* engine = seikaku_init("path/to/sde.sqlite");
//! const char* result = seikaku_calculate_eft(engine, eft_string, skills_json);
//! // use result (JSON string)
//! seikaku_free_string(result);
//! seikaku_free(engine);
//! ```

use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::calculate;
use crate::data_types;
use crate::rust::{Data, InfoMain, InfoNameMain};

/// Opaque engine handle holding the loaded SDE data.
pub struct Engine {
    data: Data,
}

/// Initialize the engine with a path to the SDE SQLite database.
///
/// Returns a pointer to the Engine, or null on failure.
/// The caller must eventually free the engine with `seikaku_free`.
#[no_mangle]
pub unsafe extern "C" fn seikaku_init(sqlite_path: *const c_char) -> *mut Engine {
    if sqlite_path.is_null() {
        return ptr::null_mut();
    }

    let path = match CStr::from_ptr(sqlite_path).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let data = Data::new(std::path::Path::new(path));
    let engine = Box::new(Engine { data });
    Box::into_raw(engine)
}

/// Calculate ship statistics from an EFT fit string.
///
/// - `engine`: pointer returned by `seikaku_init`
/// - `eft_str`: null-terminated EFT format string
/// - `skills_json`: null-terminated JSON object mapping skill type_id (string) to level (int),
///                   or null for default skills
///
/// Returns a JSON string with the calculation results.
/// The caller must free the returned string with `seikaku_free_string`.
/// Returns null on failure.
#[cfg(feature = "eft")]
#[no_mangle]
pub unsafe extern "C" fn seikaku_calculate_eft(
    engine: *const Engine,
    eft_str: *const c_char,
    skills_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() || eft_str.is_null() {
        return ptr::null_mut();
    }

    let engine = &*engine;
    let eft_text = match CStr::from_ptr(eft_str).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let info_name = InfoNameMain::new(&engine.data);

    let fit = match crate::eft::load_eft(&info_name, &eft_text) {
        Ok(f) => f.esf_fit,
        Err(_) => return ptr::null_mut(),
    };

    let skills = parse_skills(skills_json);
    calculate_and_serialize(&engine.data, fit, skills)
}

/// Calculate ship statistics from an EsfFit JSON string.
///
/// - `engine`: pointer returned by `seikaku_init`
/// - `fit_json`: null-terminated JSON string representing an `EsfFit` struct
/// - `skills_json`: null-terminated JSON object mapping skill type_id (string) to level (int),
///                   or null for default skills
///
/// Returns a JSON string with the calculation results.
/// The caller must free the returned string with `seikaku_free_string`.
/// Returns null on failure.
#[no_mangle]
pub unsafe extern "C" fn seikaku_calculate(
    engine: *const Engine,
    fit_json: *const c_char,
    skills_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() || fit_json.is_null() {
        return ptr::null_mut();
    }

    let engine = &*engine;
    let fit_text = match CStr::from_ptr(fit_json).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let fit: data_types::EsfFit = match serde_json::from_str(fit_text) {
        Ok(f) => f,
        Err(_) => return ptr::null_mut(),
    };

    let skills = parse_skills(skills_json);
    calculate_and_serialize(&engine.data, fit, skills)
}

/// Free an Engine instance created by `seikaku_init`.
#[no_mangle]
pub unsafe extern "C" fn seikaku_free(engine: *mut Engine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// Free a string returned by `seikaku_calculate` or `seikaku_calculate_eft`.
#[no_mangle]
pub unsafe extern "C" fn seikaku_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// ---------- internal helpers ----------

unsafe fn parse_skills(skills_json: *const c_char) -> BTreeMap<i32, i32> {
    let mut skills: BTreeMap<i32, i32> = BTreeMap::new();

    if !skills_json.is_null() {
        if let Ok(s) = CStr::from_ptr(skills_json).to_str() {
            if let Ok(skill_map) = serde_json::from_str::<BTreeMap<String, i32>>(s) {
                for (skill_id, level) in skill_map {
                    if let Ok(id) = skill_id.parse::<i32>() {
                        skills.insert(id, level);
                    }
                }
            }
        }
    }

    skills
}

fn calculate_and_serialize(
    data: &Data,
    fit: data_types::EsfFit,
    skills: BTreeMap<i32, i32>,
) -> *mut c_char {
    let info = InfoMain::new(fit, skills, data);
    let statistics = calculate::calculate(&info);

    match serde_json::to_string(&statistics) {
        Ok(json) => match CString::new(json) {
            Ok(c) => c.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}
