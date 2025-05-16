use std::{ffi::c_int, fmt::Display, ops::Deref, ptr::NonNull};

use super::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = ffi::LUA_OK as isize,
    Yield = ffi::LUA_YIELD as isize,
    ErrRuntime = ffi::LUA_ERRRUN as isize,
    ErrSyntax = ffi::LUA_ERRSYNTAX as isize,
    ErrMemory = ffi::LUA_ERRMEM as isize,
    ErrError = ffi::LUA_ERRERR as isize,
    Break = ffi::LUA_BREAK as isize,
}

impl From<ffi::lua_Status> for Status {
    fn from(value: ffi::lua_Status) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoroStatus {
    Run = ffi::LUA_CORUN as isize,
    Suspended = ffi::LUA_COSUS as isize,
    Normal = ffi::LUA_CONOR as isize,
    Finished = ffi::LUA_COFIN as isize,
    Errored = ffi::LUA_COERR as isize,
}

impl From<ffi::lua_CoStatus> for CoroStatus {
    fn from(value: ffi::lua_CoStatus) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    None = ffi::LUA_TNONE as isize,
    Nil = ffi::LUA_TNIL as isize,
    Boolean = ffi::LUA_TBOOLEAN as isize,
    LightUserdata = ffi::LUA_TLIGHTUSERDATA as isize,
    Number = ffi::LUA_TNUMBER as isize,
    Vector = ffi::LUA_TVECTOR as isize,
    String = ffi::LUA_TSTRING as isize,
    Table = ffi::LUA_TTABLE as isize,
    Function = ffi::LUA_TFUNCTION as isize,
    Userdata = ffi::LUA_TUSERDATA as isize,
    Thread = ffi::LUA_TTHREAD as isize,
    Buffer = ffi::LUA_TBUFFER as isize,
}

impl From<ffi::lua_Type> for Type {
    fn from(value: ffi::lua_Type) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::None => "no value",
            Self::Nil => "nil",
            Self::Boolean => "boolean",
            Self::LightUserdata => "userdata",
            Self::Number => "number",
            Self::Vector => "vector",
            Self::String => "string",
            Self::Table => "table",
            Self::Function => "function",
            Self::Userdata => "userdata",
            Self::Thread => "thread",
            Self::Buffer => "buffer",
        };

        write!(f, "{}", name)
    }
}

pub struct Ref(pub(super) Main, pub(super) c_int);

impl Drop for Ref {
    fn drop(&mut self) {
        unsafe { ffi::lua_unref(self.0.as_ptr(), self.1) }
    }
}

impl Ref {
    pub fn to_thread(&self) -> Thread {
        let stack = self.0.stack();
        stack.push_ref(self);

        let thread = stack
            .to_thread(-1)
            .expect("called to_thread on non-thread ref");

        stack.pop(1);
        Thread(thread.0)
    }
}

pub type FnReturn = c_int;

#[repr(transparent)]
pub struct Context(NonNull<ffi::lua_State>);

impl Deref for Context {
    type Target = Stack;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl Context {
    pub fn inner(&self) -> NonNull<ffi::lua_State> {
        self.0
    }

    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.0.as_ptr()
    }

    pub fn ret(self) -> FnReturn {
        0
    }

    pub fn ret_with(self, n: u32) -> FnReturn {
        n as _
    }

    pub fn yld(self) -> FnReturn {
        unsafe { ffi::lua_yield(self.as_ptr(), 0) }
    }

    pub fn yld_with(self, n: u32) -> FnReturn {
        unsafe { ffi::lua_yield(self.as_ptr(), n as _) }
    }
}
