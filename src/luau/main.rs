use std::ptr::NonNull;

use super::*;

pub struct Main(pub(super) NonNull<ffi::lua_State>);

impl Main {
    pub fn inner(&self) -> NonNull<ffi::lua_State> {
        self.0
    }

    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.0.as_ptr()
    }

    pub fn spawner<'executor>(&self) -> crate::runtime::Spawner<'executor> {
        unsafe {
            let ptr = ffi::lua_getthreaddata(self.as_ptr()) as *const crate::runtime::Spawner;

            (*ptr).clone()
        }
    }

    pub fn stack(&self) -> Stack {
        Stack(self.inner())
    }

    pub fn new_thread(&self) -> (Ref, Thread) {
        unsafe {
            let ptr = NonNull::new(ffi::lua_newthread(self.as_ptr()))
                .expect("failed to create new thread");

            let n = ffi::lua_ref(self.as_ptr(), -1);

            ffi::luaL_sandboxthread(ptr.as_ptr());
            ffi::lua_settop(self.as_ptr(), ffi::lua_gettop(self.as_ptr()) - 1);

            (Ref(Main(self.0), n), Thread(ptr))
        }
    }

    fn handle_status(&self, thread: &Thread, status: Status) {
        match status {
            Status::Ok => {}
            Status::Yield => {}

            _ => {
                let stack = thread.stack();
                let err = stack.to_string_str(-1).unwrap_or("unknown error");
                let trace = unsafe {
                    let ptr = ffi::lua_debugtrace(thread.as_ptr());
                    std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(ptr).to_bytes())
                };

                eprint!("{err}\ntraceback:\n{trace}");
            }
        }
    }

    pub fn spawn(&self, thread: &Thread, nargs: u32) {
        self.handle_status(thread, thread.resume(None, nargs));
    }

    pub fn spawn_error(&self, thread: &Thread) {
        self.handle_status(thread, thread.resume_error(None));
    }
}
