use std::ptr::{NonNull, null_mut};

use super::*;

#[repr(transparent)]
pub struct Thread(pub(super) NonNull<ffi::lua_State>);

impl Thread {
    pub fn inner(&self) -> NonNull<ffi::lua_State> {
        self.0
    }

    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> Main {
        Main(unsafe { NonNull::new_unchecked(ffi::lua_mainthread(self.as_ptr())) })
    }

    pub fn stack(&self) -> Stack {
        Stack(self.0)
    }

    pub fn resume(&self, from: Option<&Thread>, n: u32) -> Status {
        unsafe {
            Status::from(ffi::lua_resume(
                self.as_ptr(),
                from.map(|t| t.as_ptr()).unwrap_or(null_mut()),
                n as _,
            ))
        }
    }

    pub fn resume_error(&self, from: Option<&Thread>) -> Status {
        unsafe {
            Status::from(ffi::lua_resumeerror(
                self.as_ptr(),
                from.map(|t| t.as_ptr()).unwrap_or(null_mut()),
            ))
        }
    }

    pub fn status(&self) -> Status {
        unsafe { Status::from(ffi::lua_status(self.as_ptr())) }
    }

    pub fn coro_status(&self) -> CoroStatus {
        unsafe { CoroStatus::from(ffi::lua_costatus(self.as_ptr())) }
    }

    pub fn to_ref(&self) -> Ref {
        let stack = self.stack();
        stack.push_thread(self);
        let r = stack.to_ref(-1);
        stack.pop(1);
        r
    }
}
