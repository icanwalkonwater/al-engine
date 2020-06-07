use std::ffi::CStr;
use std::os::raw::c_char;

#[inline]
pub(crate) fn vk_to_owned_string(raw: &[c_char]) -> String {
    let wrapped = unsafe { CStr::from_ptr(raw.as_ptr()) };

    wrapped.to_string_lossy().into_owned()
}

#[inline]
pub(crate) fn str_to_cstr(str: &str) -> &CStr {
    unsafe { CStr::from_ptr(str.as_ptr() as *mut _) }
}
