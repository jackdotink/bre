use std::{ffi::CString, path::PathBuf, ptr::NonNull};

use super::*;

pub struct Main(pub(super) NonNull<ffi::lua_State>);

impl Main {
    pub fn inner(&self) -> NonNull<ffi::lua_State> {
        self.0
    }

    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.0.as_ptr()
    }

    fn data<'executor>(&self) -> &LuauData<'executor> {
        unsafe { &*(ffi::lua_getthreaddata(self.as_ptr()) as *const LuauData<'executor>) }
    }

    pub fn spawner<'executor>(&self) -> crate::runtime::Spawner<'executor> {
        self.data().spawner.clone()
    }

    pub fn compiler(&self) -> Compiler {
        self.data().compiler.clone()
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

    pub fn handle_status(&self, thread: &Thread, status: Status) {
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

    pub fn execute(&self, path: PathBuf, bytecode: &Bytecode) -> (Status, Stack) {
        let (_, thread) = self.new_thread();
        let stack = thread.stack();

        stack.push_bytecode(
            &unsafe { CString::from_vec_unchecked(path.display().to_string().into_bytes()) },
            bytecode,
        );

        (thread.resume(None, 0), stack)
    }
}
