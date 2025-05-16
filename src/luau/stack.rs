use std::{
    ffi::{CStr, c_void},
    ptr::NonNull,
};

use super::*;

pub struct Stack(pub(super) NonNull<ffi::lua_State>);

impl Stack {
    pub fn inner(&self) -> NonNull<ffi::lua_State> {
        self.0
    }

    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> Main {
        Main(unsafe { NonNull::new_unchecked(ffi::lua_mainthread(self.as_ptr())) })
    }

    pub fn spawner<'executor>(&self) -> crate::runtime::Spawner<'executor> {
        self.main().spawner()
    }

    pub fn thread(&self) -> Thread {
        Thread(self.0)
    }

    pub fn error(&self, msg: impl AsRef<[u8]>) -> ! {
        self.push_string(msg);
        unsafe { ffi::lua_error(self.as_ptr()) }
    }

    pub fn get_top(&self) -> u32 {
        unsafe { ffi::lua_gettop(self.as_ptr()) as _ }
    }

    pub fn set_top(&self, n: u32) {
        unsafe { ffi::lua_settop(self.as_ptr(), n as _) }
    }

    pub fn pop(&self, n: u32) {
        self.set_top(self.get_top() - n);
    }

    pub fn xmove(&self, to: &Thread, n: u32) {
        unsafe { ffi::lua_xmove(self.as_ptr(), to.as_ptr(), n as _) };
    }

    pub fn xpush(&self, to: &Thread, n: u32) {
        unsafe { ffi::lua_xpush(self.as_ptr(), to.as_ptr(), n as _) };
    }

    pub fn remove(&self, idx: i32) {
        unsafe { ffi::lua_remove(self.as_ptr(), idx as _) };
    }

    pub fn insert(&self, idx: i32) {
        unsafe { ffi::lua_insert(self.as_ptr(), idx as _) };
    }

    pub fn replace(&self, idx: i32) {
        unsafe { ffi::lua_replace(self.as_ptr(), idx as _) };
    }

    pub fn check(&self, n: u32) {
        let result = unsafe { ffi::lua_checkstack(self.as_ptr(), n as _) };
        assert!(result != 0, "stack overflow");
    }

    pub fn push_copy(&self, idx: i32) {
        unsafe { ffi::lua_pushvalue(self.as_ptr(), idx) };
    }

    pub fn push_nil(&self) {
        unsafe { ffi::lua_pushnil(self.as_ptr()) };
    }

    pub fn push_boolean(&self, b: bool) {
        unsafe { ffi::lua_pushboolean(self.as_ptr(), b as _) };
    }

    pub fn push_light_userdata(&self, p: *mut c_void) {
        unsafe { ffi::lua_pushlightuserdata(self.as_ptr(), p) };
    }

    pub fn push_number(&self, n: f64) {
        unsafe { ffi::lua_pushnumber(self.as_ptr(), n) };
    }

    pub fn push_vector(&self, v: (f32, f32, f32)) {
        unsafe { ffi::lua_pushvector(self.as_ptr(), v.0, v.1, v.2) };
    }

    pub fn push_string(&self, s: impl AsRef<[u8]>) {
        let s = s.as_ref();
        unsafe { ffi::lua_pushlstring(self.as_ptr(), s.as_ptr() as _, s.len()) };
    }

    pub fn push_table(&self) {
        unsafe { ffi::lua_createtable(self.as_ptr(), 0, 0) };
    }

    pub fn push_table_with(&self, narr: u32, nrec: u32) {
        unsafe { ffi::lua_createtable(self.as_ptr(), narr as _, nrec as _) };
    }

    pub fn push_function(
        &self,
        name: &'static CStr,
        func: extern "C-unwind" fn(ctx: Context) -> FnReturn,
    ) {
        unsafe {
            let func = std::mem::transmute::<
                extern "C-unwind" fn(ctx: Context) -> FnReturn,
                extern "C-unwind" fn(*mut ffi::lua_State) -> FnReturn,
            >(func);

            ffi::lua_pushcclosurek(self.as_ptr(), func, name.as_ptr() as _, 0, None);
        }
    }

    pub fn push_bytecode(&self, name: &CStr, bytecode: &Bytecode) {
        unsafe {
            ffi::luau_load(
                self.as_ptr(),
                name.as_ptr() as _,
                bytecode.inner().as_ptr() as _,
                bytecode.inner().len() as _,
                0,
            );
        }
    }

    pub fn push_thread(&self, thread: &Thread) {
        if thread.as_ptr() == self.as_ptr() {
            unsafe {
                ffi::lua_pushthread(self.as_ptr());
            }
        } else {
            let stack = thread.stack();
            stack.check(1);
            unsafe { ffi::lua_pushthread(stack.as_ptr()) };
            stack.xmove(&self.thread(), 1);
        }
    }

    pub fn push_thread_new(&self) -> Thread {
        unsafe {
            let ptr = NonNull::new(ffi::lua_newthread(self.as_ptr()))
                .expect("failed to create new thread");

            ffi::luaL_sandboxthread(ptr.as_ptr());

            Thread(ptr)
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn push_buffer(&self, size: usize) -> &mut [u8] {
        let ptr = unsafe { ffi::lua_newbuffer(self.as_ptr(), size) };
        unsafe { std::slice::from_raw_parts_mut(ptr as _, size) }
    }

    pub fn push_ref(&self, r: &Ref) {
        unsafe { ffi::lua_rawgeti(self.as_ptr(), ffi::LUA_REGISTRYINDEX, r.1) };
    }

    pub fn type_of(&self, idx: i32) -> Type {
        unsafe { Type::from(ffi::lua_type(self.as_ptr(), idx as _)) }
    }

    pub fn is_none(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::None
    }

    pub fn is_nil(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Nil
    }

    pub fn is_boolean(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Boolean
    }

    pub fn is_light_userdata(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::LightUserdata
    }

    pub fn is_number(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Number
    }

    pub fn is_vector(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Vector
    }

    pub fn is_string(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::String
    }

    pub fn is_table(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Table
    }

    pub fn is_function(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Function
    }

    pub fn is_userdata(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Userdata
    }

    pub fn is_thread(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Thread
    }

    pub fn is_buffer(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Buffer
    }

    pub fn to_boolean(&self, idx: i32) -> Option<bool> {
        if self.is_boolean(idx) {
            Some(unsafe { ffi::lua_toboolean(self.as_ptr(), idx as _) != 0 })
        } else {
            None
        }
    }

    pub fn to_light_userdata(&self, idx: i32) -> Option<*mut c_void> {
        if self.is_light_userdata(idx) {
            Some(unsafe { ffi::lua_touserdata(self.as_ptr(), idx as _) })
        } else {
            None
        }
    }

    pub fn to_number(&self, idx: i32) -> Option<f64> {
        let mut isnum = 0;
        let num = unsafe { ffi::lua_tonumberx(self.as_ptr(), idx, &mut isnum) };

        if isnum != 0 { Some(num) } else { None }
    }

    pub fn to_vector(&self, idx: i32) -> Option<(f32, f32, f32)> {
        let ptr = unsafe { ffi::lua_tovector(self.as_ptr(), idx) };

        if !ptr.is_null() {
            Some(unsafe { (ptr.read(), ptr.add(1).read(), ptr.add(2).read()) })
        } else {
            None
        }
    }

    pub fn to_string_slice(&self, idx: i32) -> Option<&[u8]> {
        let mut len = 0;
        let ptr = unsafe { ffi::lua_tolstring(self.as_ptr(), idx, &mut len) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr as _, len) })
        } else {
            None
        }
    }

    pub fn to_string_str(&self, idx: i32) -> Option<&str> {
        std::str::from_utf8(self.to_string_slice(idx)?).ok()
    }

    pub fn to_thread(&self, idx: i32) -> Option<Thread> {
        let ptr = unsafe { ffi::lua_tothread(self.as_ptr(), idx as _) };

        if !ptr.is_null() {
            Some(Thread(unsafe { NonNull::new_unchecked(ptr) }))
        } else {
            None
        }
    }

    pub fn to_buffer(&self, idx: i32) -> Option<&mut [u8]> {
        let mut len = 0;
        let ptr = unsafe { ffi::lua_tobuffer(self.as_ptr(), idx, &mut len) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts_mut(ptr as _, len) })
        } else {
            None
        }
    }

    pub fn to_ref(&self, idx: i32) -> Ref {
        Ref(self.main(), unsafe {
            ffi::lua_ref(self.as_ptr(), idx as _)
        })
    }

    pub fn arg_boolean(&self, idx: u32) -> bool {
        if let Some(b) = self.to_boolean(idx as _) {
            b
        } else {
            self.error(format!(
                "bad argument #{idx} to function (boolean expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_boolean_opt(&self, idx: u32) -> Option<bool> {
        if let Some(b) = self.to_boolean(idx as _) {
            Some(b)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (boolean or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn arg_number(&self, idx: u32) -> f64 {
        if let Some(n) = self.to_number(idx as _) {
            n
        } else {
            self.error(format!(
                "bad argument #{idx} to function (number expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_number_opt(&self, idx: u32) -> Option<f64> {
        if let Some(n) = self.to_number(idx as _) {
            Some(n)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (number or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn arg_vector(&self, idx: u32) -> (f32, f32, f32) {
        if let Some(v) = self.to_vector(idx as _) {
            v
        } else {
            self.error(format!(
                "bad argument #{idx} to function (vector expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_vector_opt(&self, idx: u32) -> Option<(f32, f32, f32)> {
        if let Some(v) = self.to_vector(idx as _) {
            Some(v)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (vector or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn arg_string_slice(&self, idx: u32) -> &[u8] {
        if let Some(s) = self.to_string_slice(idx as _) {
            s
        } else {
            self.error(format!(
                "bad argument #{idx} to function (string expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_string_str(&self, idx: u32) -> &str {
        if let Some(s) = self.to_string_str(idx as _) {
            s
        } else {
            self.error(format!(
                "bad argument #{idx} to function (string expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_string_opt_slice(&self, idx: u32) -> Option<&[u8]> {
        if let Some(s) = self.to_string_slice(idx as _) {
            Some(s)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (string or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn arg_string_opt_str(&self, idx: u32) -> Option<&str> {
        if let Some(s) = self.to_string_str(idx as _) {
            Some(s)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (string or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn arg_table(&self, idx: u32) {
        match self.type_of(idx as _) {
            Type::Table => (),

            ty => self.error(format!(
                "bad argument #{idx} to function (table expected, got {})",
                ty
            )),
        }
    }

    pub fn arg_table_opt(&self, idx: u32) -> Option<()> {
        match self.type_of(idx as _) {
            Type::Table => Some(()),
            Type::None | Type::Nil => None,

            ty => self.error(format!(
                "bad argument #{idx} to function (table or nil expected, got {})",
                ty
            )),
        }
    }

    pub fn arg_thread(&self, idx: u32) -> Thread {
        match self.type_of(idx as _) {
            Type::Thread => self.to_thread(idx as _).unwrap(),

            ty => self.error(format!(
                "bad argument #{idx} to function (thread expected, got {})",
                ty
            )),
        }
    }

    pub fn arg_thread_opt(&self, idx: u32) -> Option<Thread> {
        match self.type_of(idx as _) {
            Type::Thread => self.to_thread(idx as _),

            Type::None | Type::Nil => None,

            ty => self.error(format!(
                "bad argument #{idx} to function (thread or nil expected, got {})",
                ty
            )),
        }
    }

    pub fn arg_buffer(&self, idx: u32) -> &mut [u8] {
        if let Some(b) = self.to_buffer(idx as _) {
            b
        } else {
            self.error(format!(
                "bad argument #{idx} to function (buffer expected, got {})",
                self.type_of(idx as _)
            ))
        }
    }

    pub fn arg_buffer_opt(&self, idx: u32) -> Option<&mut [u8]> {
        if let Some(b) = self.to_buffer(idx as _) {
            Some(b)
        } else {
            match self.type_of(idx as _) {
                Type::None | Type::Nil => None,

                ty => self.error(format!(
                    "bad argument #{idx} to function (buffer or nil expected, got {})",
                    ty
                )),
            }
        }
    }

    pub fn table_get(&self, tbl_idx: i32) {
        unsafe { ffi::lua_gettable(self.as_ptr(), tbl_idx as _) };
    }

    pub fn table_set(&self, tbl_idx: i32) {
        unsafe { ffi::lua_settable(self.as_ptr(), tbl_idx as _) };
    }

    pub fn table_get_field(&self, tbl_idx: i32, key: &CStr) {
        unsafe { ffi::lua_getfield(self.as_ptr(), tbl_idx as _, key.as_ptr() as _) };
    }

    pub fn table_set_field(&self, tbl_idx: i32, key: &CStr) {
        unsafe { ffi::lua_setfield(self.as_ptr(), tbl_idx as _, key.as_ptr() as _) };
    }

    pub fn table_get_raw(&self, tbl_idx: i32) {
        unsafe { ffi::lua_rawget(self.as_ptr(), tbl_idx as _) };
    }

    pub fn table_set_raw(&self, tbl_idx: i32) {
        unsafe { ffi::lua_rawset(self.as_ptr(), tbl_idx as _) };
    }

    pub fn table_get_raw_i(&self, tbl_idx: i32, key: u32) {
        unsafe { ffi::lua_rawgeti(self.as_ptr(), tbl_idx as _, key as _) };
    }

    pub fn table_set_raw_i(&self, tbl_idx: i32, key: u32) {
        unsafe { ffi::lua_rawseti(self.as_ptr(), tbl_idx as _, key as _) };
    }

    pub fn table_get_raw_field(&self, tbl_idx: i32, key: &CStr) {
        unsafe { ffi::lua_rawgetfield(self.as_ptr(), tbl_idx as _, key.as_ptr() as _) };
    }

    pub fn table_set_raw_field(&self, tbl_idx: i32, key: &CStr) {
        unsafe { ffi::lua_rawsetfield(self.as_ptr(), tbl_idx as _, key.as_ptr() as _) };
    }
}
