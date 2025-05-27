#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_double, c_float, c_int, c_uchar, c_void};
use std::marker::{PhantomData, PhantomPinned};

pub const LUA_MULTRET: c_int = -1;

pub const LUA_REGISTRYINDEX: c_int = -8000 - 2000;
pub const LUA_ENVIRONINDEX: c_int = -8000 - 2001;
pub const LUA_GLOBALSINDEX: c_int = -8000 - 2002;

pub const fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}

pub const fn lua_ispseudo(i: c_int) -> bool {
    i <= LUA_REGISTRYINDEX
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum lua_Status {
    LUA_OK = 0,
    LUA_YIELD,
    LUA_ERRRUN,
    LUA_ERRSYNTAX,
    LUA_ERRMEM,
    LUA_ERRERR,
    LUA_BREAK,
}

pub use lua_Status::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum lua_CoStatus {
    LUA_CORUN = 0,
    LUA_COSUS,
    LUA_CONOR,
    LUA_COFIN,
    LUA_COERR,
}

pub use lua_CoStatus::*;

#[repr(C)]
#[derive(Debug)]
pub struct lua_State {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

pub type lua_CFunction = unsafe extern "C-unwind" fn(L: *mut lua_State) -> c_int;
pub type lua_Continuation = unsafe extern "C-unwind" fn(L: *mut lua_State, status: c_int) -> c_int;

pub type lua_Alloc = unsafe extern "C-unwind" fn(
    ud: *mut c_void,
    ptr: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum lua_Type {
    LUA_TNONE = -1,
    LUA_TNIL = 0,
    LUA_TBOOLEAN = 1,

    LUA_TLIGHTUSERDATA,
    LUA_TNUMBER,
    LUA_TVECTOR,

    LUA_TSTRING,

    LUA_TTABLE,
    LUA_TFUNCTION,
    LUA_TUSERDATA,
    LUA_TTHREAD,
    LUA_TBUFFER,
}

pub use lua_Type::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct lua_Debug {
    pub name: *const c_char,
    pub what: *const c_char,
    pub source: *const c_char,
    pub short_src: *const c_char,
    pub linedefined: c_int,
    pub currentline: c_int,
    pub nupvals: c_uchar,
    pub nparams: c_uchar,
    pub isvararg: c_char,
    pub userdata: *mut c_void,

    pub ssbuf: [c_char; 256],
}

unsafe extern "C-unwind" {
    pub fn lua_newstate(f: lua_Alloc, ud: *mut c_void) -> *mut lua_State;
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_newthread(L: *mut lua_State) -> *mut lua_State;
    pub fn lua_mainthread(L: *mut lua_State) -> *mut lua_State;

    pub fn lua_getinfo(
        L: *mut lua_State,
        level: c_int,
        what: *const c_char,
        ar: *mut lua_Debug,
    ) -> c_int;

    pub fn luau_load(
        L: *mut lua_State,
        chunkname: *const c_char,
        data: *const c_char,
        size: usize,
        env: c_int,
    ) -> c_int;

    pub fn lua_pcall(
        L: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
    ) -> lua_Status;

    pub fn lua_error(L: *mut lua_State) -> !;

    pub fn lua_resume(L: *mut lua_State, from: *mut lua_State, narg: c_int) -> lua_Status;
    pub fn lua_resumeerror(L: *mut lua_State, from: *mut lua_State) -> lua_Status;
    pub fn lua_yield(L: *mut lua_State, nresults: c_int) -> c_int;
    pub fn lua_status(L: *mut lua_State) -> lua_Status;
    pub fn lua_costatus(L: *mut lua_State) -> lua_CoStatus;
    pub fn lua_getthreaddata(L: *mut lua_State) -> *mut c_void;
    pub fn lua_setthreaddata(L: *mut lua_State, data: *mut c_void);

    pub fn lua_gettop(L: *mut lua_State) -> c_int;
    pub fn lua_settop(L: *mut lua_State, idx: c_int);
    pub fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int);
    pub fn lua_xpush(from: *mut lua_State, to: *mut lua_State, idx: c_int);
    pub fn lua_remove(L: *mut lua_State, idx: c_int);
    pub fn lua_insert(L: *mut lua_State, idx: c_int);
    pub fn lua_replace(L: *mut lua_State, idx: c_int);

    pub fn lua_checkstack(L: *mut lua_State, sz: c_int) -> c_int;

    pub fn lua_debugtrace(L: *mut lua_State) -> *const c_char;

    pub fn lua_pushvalue(L: *mut lua_State, idx: c_int);
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushboolean(L: *mut lua_State, b: c_int);
    pub fn lua_pushlightuserdatatagged(L: *mut lua_State, p: *mut c_void, tag: c_int);
    pub fn lua_pushnumber(L: *mut lua_State, n: c_double);
    pub fn lua_pushvector(L: *mut lua_State, x: c_float, y: c_float, z: c_float);
    pub fn lua_pushlstring(L: *mut lua_State, s: *const c_char, l: usize);
    pub fn lua_pushthread(L: *mut lua_State) -> c_int;

    pub fn lua_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_gettable(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_settable(L: *mut lua_State, idx: c_int);
    pub fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int;
    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char);
    pub fn lua_rawget(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_rawset(L: *mut lua_State, idx: c_int);
    pub fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: c_int) -> c_int;
    pub fn lua_rawseti(L: *mut lua_State, idx: c_int, n: c_int);
    pub fn lua_rawgetfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int;
    pub fn lua_rawsetfield(L: *mut lua_State, idx: c_int, k: *const c_char);
    pub fn lua_getmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_setmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_getreadonly(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_setreadonly(L: *mut lua_State, idx: c_int, enabled: c_int);

    pub fn lua_objlen(L: *mut lua_State, idx: c_int) -> usize;

    pub fn lua_newbuffer(L: *mut lua_State, size: usize) -> *mut c_void;

    pub fn lua_pushcclosurek(
        L: *mut lua_State,
        r#fn: lua_CFunction,
        debugname: *const c_char,
        nup: c_int,
        cont: Option<lua_Continuation>,
    );

    pub fn lua_ref(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_unref(L: *mut lua_State, r#ref: c_int);

    pub fn lua_type(L: *mut lua_State, idx: c_int) -> lua_Type;
    pub fn lua_tonumberx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> c_double;
    pub fn lua_tovector(L: *mut lua_State, idx: c_int) -> *const c_float;
    pub fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;
    pub fn lua_tolightuserdata(L: *mut lua_State, idx: c_int) -> *mut c_void;
    pub fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut c_void;
    pub fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State;
    pub fn lua_tobuffer(L: *mut lua_State, idx: c_int, len: *mut usize) -> *mut c_void;
}
