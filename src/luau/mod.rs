use std::{ffi::CString, path::Path, ptr::NonNull};

pub mod ffi;

mod compiler;
mod extra;
mod library;
mod main;
mod stack;
mod thread;
mod userdata;

pub use compiler::{Bytecode, Compiler};
pub use extra::*;
pub use library::*;
pub use main::Main;
pub use stack::Stack;
pub use thread::Thread;
pub use userdata::*;

pub const REGISTRY_IDX: i32 = ffi::LUA_REGISTRYINDEX;

struct LuauData<'executor> {
    spawner: crate::runtime::Spawner<'executor>,
    compiler: Compiler,
}

pub struct Luau<'executor> {
    state: NonNull<ffi::lua_State>,
    data: *const LuauData<'executor>,
}

impl<'executor> Luau<'executor> {
    pub fn new(spawner: crate::runtime::Spawner<'executor>, compiler: Compiler) -> Self {
        #[allow(unused)]
        unsafe extern "C-unwind" fn lua_alloc(
            _ud: *mut std::ffi::c_void,
            ptr: *mut std::ffi::c_void,
            osize: usize,
            nsize: usize,
        ) -> *mut std::ffi::c_void {
            if nsize == 0 {
                unsafe { libc::free(ptr) };
                std::ptr::null_mut()
            } else {
                unsafe { libc::realloc(ptr, nsize) }
            }
        }

        let data = Box::into_raw(Box::new(LuauData { spawner, compiler }));
        let state = NonNull::new(unsafe { ffi::lua_newstate(lua_alloc, std::ptr::null_mut()) })
            .expect("failed to create lua state");

        let luau = Self { state, data };

        unsafe {
            ffi::lua_setthreaddata(state.as_ptr(), data as _);

            ffi::luaopen_base(state.as_ptr());
            ffi::luaopen_coroutine(state.as_ptr());
            ffi::luaopen_table(state.as_ptr());
            ffi::luaopen_os(state.as_ptr());
            ffi::luaopen_string(state.as_ptr());
            ffi::luaopen_bit32(state.as_ptr());
            ffi::luaopen_buffer(state.as_ptr());
            ffi::luaopen_utf8(state.as_ptr());
            ffi::luaopen_math(state.as_ptr());
            ffi::luaopen_debug(state.as_ptr());
            ffi::luaopen_vector(state.as_ptr());

            crate::globals::require::open(&Main(state));

            crate::globals::task::Task::push(Stack(state));
            ffi::lua_setfield(state.as_ptr(), ffi::LUA_GLOBALSINDEX, c"task".as_ptr());

            ffi::luaL_sandbox(state.as_ptr());
        }

        luau
    }

    pub fn execute(&self, path: &Path, bytecode: &Bytecode) {
        let main = Main(self.state);

        let (_, thread) = main.new_thread();
        let stack = thread.stack();

        let name = CString::new(path.to_str().unwrap()).unwrap();

        stack.push_bytecode(name.as_c_str(), bytecode);
        main.spawn(&thread, 0);
    }
}

impl Drop for Luau<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::lua_close(self.state.as_ptr());
            drop(Box::from_raw(self.data as *mut LuauData));
        }
    }
}
