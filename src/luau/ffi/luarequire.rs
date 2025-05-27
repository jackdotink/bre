#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

use super::lua_State;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum luarequire_NavigateResult {
    NAVIGATE_SUCCESS,
    NAVIGATE_AMBIGUOUS,
    NAVIGATE_NOT_FOUND,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum luarequire_WriteResult {
    WRITE_SUCCESS,
    WRITE_BUFFER_TOO_SMALL,
    WRITE_FAILURE,
}

pub type luarequire_configuration_init =
    extern "C-unwind" fn(config: *mut luarequire_Configuration);

#[repr(C)]
#[derive(Debug)]
pub struct luarequire_Configuration {
    pub is_require_allowed: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        requirer_chunkname: *const c_char,
    ) -> bool,

    pub reset: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        requirer_chunkname: *const c_char,
    ) -> luarequire_NavigateResult,

    pub jump_to_alias: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        path: *const c_char,
    ) -> luarequire_NavigateResult,

    pub to_parent:
        extern "C-unwind" fn(L: *mut lua_State, ctx: *mut c_void) -> luarequire_NavigateResult,

    pub to_child: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        name: *const c_char,
    ) -> luarequire_NavigateResult,

    pub is_module_present: extern "C-unwind" fn(L: *mut lua_State, ctx: *mut c_void) -> bool,

    pub get_chunkname: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        buffer: *mut c_char,
        buffer_size: usize,
        size_out: *mut usize,
    ) -> luarequire_WriteResult,

    pub get_loadname: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        buffer: *mut c_char,
        buffer_size: usize,
        size_out: *mut usize,
    ) -> luarequire_WriteResult,

    pub get_cache_key: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        buffer: *mut c_char,
        buffer_size: usize,
        size_out: *mut usize,
    ) -> luarequire_WriteResult,

    pub is_config_present: extern "C-unwind" fn(L: *mut lua_State, ctx: *mut c_void) -> bool,

    pub get_alias: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        alias: *const c_char,
        buffer: *mut c_char,
        buffer_size: usize,
        size_out: *mut usize,
    ) -> luarequire_WriteResult,

    pub get_config: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        buffer: *mut c_char,
        buffer_size: usize,
        size_out: *mut usize,
    ) -> luarequire_WriteResult,

    pub load: extern "C-unwind" fn(
        L: *mut lua_State,
        ctx: *mut c_void,
        path: *const c_char,
        chunkname: *const c_char,
        loadname: *const c_char,
    ) -> c_int,
}

unsafe extern "C-unwind" {
    pub fn luaopen_require(
        L: *mut lua_State,
        config_init: luarequire_configuration_init,
        ctx: *mut c_void,
    );
}
