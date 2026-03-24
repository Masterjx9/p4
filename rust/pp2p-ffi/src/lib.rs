use pp2p_core::{
    envelope_from_json, envelope_to_json, generate_identity, peer_id_from_public_key_b64,
    sign_envelope, signing_key_from_b64, verifying_key_from_b64, CoreError,
};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};
use std::sync::Mutex;

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(msg);
    }
}

fn clear_last_error() {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
}

fn read_cstr(ptr: *const c_char, field: &'static str) -> Result<String, CoreError> {
    if ptr.is_null() {
        return Err(CoreError::Json(format!("{field} pointer is null")));
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_string_lossy().into_owned())
}

fn into_c_string_ptr(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(c) => c.into_raw(),
        Err(e) => {
            set_last_error(format!("failed to create C string: {e}"));
            std::ptr::null_mut()
        }
    }
}

fn with_error_capture<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Result<T, CoreError>,
{
    clear_last_error();
    match f() {
        Ok(v) => Some(v),
        Err(e) => {
            set_last_error(e.to_string());
            None
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_generate_identity_json() -> *mut c_char {
    match with_error_capture(|| {
        let identity = generate_identity();
        serde_json::to_string(&identity).map_err(|e| CoreError::Json(e.to_string()))
    }) {
        Some(json) => into_c_string_ptr(json),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_peer_id_from_public_key_b64(public_key_b64: *const c_char) -> *mut c_char {
    match with_error_capture(|| {
        let key = read_cstr(public_key_b64, "public_key_b64")?;
        peer_id_from_public_key_b64(&key)
    }) {
        Some(peer_id) => into_c_string_ptr(peer_id),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_sign_envelope_json(
    private_key_b64: *const c_char,
    sender_peer_id: *const c_char,
    recipient_peer_id: *const c_char,
    payload_json: *const c_char,
    timestamp_ms: u64,
    nonce: *const c_char,
) -> *mut c_char {
    match with_error_capture(|| {
        let private_key = read_cstr(private_key_b64, "private_key_b64")?;
        let sender = read_cstr(sender_peer_id, "sender_peer_id")?;
        let recipient = read_cstr(recipient_peer_id, "recipient_peer_id")?;
        let payload_raw = read_cstr(payload_json, "payload_json")?;
        let nonce = read_cstr(nonce, "nonce")?;

        let signing_key = signing_key_from_b64(&private_key)?;
        let payload: Value =
            serde_json::from_str(&payload_raw).map_err(|e| CoreError::Json(e.to_string()))?;
        let envelope = sign_envelope(
            &signing_key,
            &sender,
            &recipient,
            timestamp_ms,
            &nonce,
            payload,
        )?;
        envelope_to_json(&envelope)
    }) {
        Some(json) => into_c_string_ptr(json),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_verify_envelope_json(
    envelope_json: *const c_char,
    signer_public_key_b64: *const c_char,
    now_ms: u64,
    max_skew_ms: u64,
) -> c_uchar {
    match with_error_capture(|| {
        let envelope_raw = read_cstr(envelope_json, "envelope_json")?;
        let signer_key_raw = read_cstr(signer_public_key_b64, "signer_public_key_b64")?;
        let envelope = envelope_from_json(&envelope_raw)?;
        let verifying_key = verifying_key_from_b64(&signer_key_raw)?;
        pp2p_core::verify_envelope(&envelope, &verifying_key, now_ms, max_skew_ms)?;
        Ok(())
    }) {
        Some(()) => 1,
        None => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_last_error_message() -> *mut c_char {
    let msg = LAST_ERROR
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_default();
    into_c_string_ptr(msg)
}

#[unsafe(no_mangle)]
pub extern "C" fn pp2p_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

