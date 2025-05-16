#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

pub type lua_CompileConstant = *mut c_void;

pub type lua_LibraryMemberTypeCallback =
    unsafe extern "C-unwind" fn(library: *const c_char, member: *const c_char);

pub type lua_LibraryMemberConstantCallback = unsafe extern "C-unwind" fn(
    library: *const c_char,
    member: *const c_char,
    constant: *mut lua_CompileConstant,
);

#[repr(C)]
#[derive(Debug)]
pub struct lua_CompileOptions {
    pub optimizationLevel: c_int,
    pub debugLevel: c_int,
    pub typeInfoLevel: c_int,
    pub coverageLevel: c_int,
    pub vectorLib: *const c_char,
    pub vectorCtor: *const c_char,
    pub vectorType: *const c_char,
    pub mutableGlobals: *const *const c_char,
    pub userdataTypes: *const *const c_char,
    pub librariesWithKnownMembers: *const *const c_char,
    pub libraryMemberTypeCb: Option<lua_LibraryMemberTypeCallback>,
    pub libraryMemberConstantCb: Option<lua_LibraryMemberConstantCallback>,
    pub disabledBuiltins: *const *const c_char,
}

unsafe extern "C-unwind" {
    pub fn luau_compile(
        source: *const c_char,
        size: usize,
        options: *mut lua_CompileOptions,
        outsize: *mut usize,
    ) -> *mut c_char;
}
