use alloc::{borrow::Cow, boxed::Box};
use core::{
    ffi::{c_char, CStr},
    ptr, slice,
};

#[repr(C)]
pub struct Buffer {
    ptr: *const u8,
    len: usize,

    /// IGNORE
    __is_owned: bool,
}

#[inline(always)]
unsafe fn __bind_fn(ptr: *const c_char, f: fn(&str) -> crate::Result<Cow<str>>) -> Buffer {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    if let Ok(s) = cstr.to_str() {
        if let Ok(d) = f(s) {
            match d {
                Cow::Owned(mut own) => {
                    own.push('\0');
                    let own = own.into_boxed_str();
                    let len = own.len();
                    return Buffer {
                        ptr: Box::into_raw(own) as *const u8,
                        len,
                        __is_owned: true,
                    };
                }
                Cow::Borrowed(bor) => {
                    return Buffer {
                        ptr: bor.as_ptr(),
                        len: bor.len(),
                        __is_owned: false,
                    }
                }
            }
        }
    }
    Buffer {
        ptr: ptr::null(),
        len: 0,
        __is_owned: true,
    }
}

/// Decodes the given string
///
/// Buffer must be free'd with `url_buffer_free`
///
/// # Safety
/// ptr must be a valid null-terminated C-string
#[no_mangle]
pub unsafe extern "C" fn url_decode(ptr: *const c_char) -> Buffer {
    __bind_fn(ptr, crate::decode)
}

/// Encodes the given string
///
/// Buffer must be free'd with `url_buffer_free`
///
/// # Safety
/// ptr must be a valid null-terminated C-string
#[no_mangle]
pub unsafe extern "C" fn url_encode(ptr: *const c_char) -> Buffer {
    __bind_fn(ptr, crate::encode)
}

/// Frees the given buffer
#[no_mangle]
pub extern "C" fn url_buffer_free(ptr: Buffer) {
    if ptr.__is_owned && !ptr.ptr.is_null() {
        let slice = unsafe { slice::from_raw_parts_mut(ptr.ptr as *mut u8, ptr.len) };
        let b = unsafe { Box::from_raw(slice) };
        drop(b);
    }
}
