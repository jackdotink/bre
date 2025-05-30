use std::{
    ffi::{CStr, c_char, c_int, c_void},
    io::Read,
    path::{Path, PathBuf},
};

use crate::luau;

fn ptr_to_str(stack: &luau::Stack, ptr: *const c_char) -> &str {
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .unwrap_or_else(|_| stack.push_error("non-utf8 string"))
    }
}

enum Reason {
    NotFound,
    Ambiguous,
}

impl From<Result<(), Reason>> for luau::ffi::luarequire_NavigateResult {
    fn from(value: Result<(), Reason>) -> Self {
        match value {
            Ok(()) => luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS,
            Err(Reason::NotFound) => luau::ffi::luarequire_NavigateResult::NAVIGATE_NOT_FOUND,
            Err(Reason::Ambiguous) => luau::ffi::luarequire_NavigateResult::NAVIGATE_AMBIGUOUS,
        }
    }
}

#[repr(transparent)]
struct Current(*mut c_void);

impl Current {
    pub fn as_ptr(&self) -> *mut PathBuf {
        self.0 as _
    }

    pub fn as_path(&self) -> &Path {
        unsafe { self.as_ptr().as_ref().unwrap_unchecked() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_pathbuf(&self) -> &mut PathBuf {
        unsafe { self.as_ptr().as_mut().unwrap_unchecked() }
    }

    pub fn as_str(&self) -> &str {
        self.as_path().to_str().expect("path is not valid utf8")
    }

    pub fn possible_paths(&self) -> (PathBuf, PathBuf) {
        (
            self.as_path().with_extension("luau"),
            self.as_path().join("init.luau"),
        )
    }

    pub fn exists(&self) -> bool {
        let (first, second) = self.possible_paths();

        first.is_file() || second.is_file()
    }

    pub fn is_ambiguous(&self) -> bool {
        let (first, second) = self.possible_paths();

        first.is_file() && second.is_file()
    }

    pub fn jump(&self, path: &str) -> Result<(), Reason> {
        match Path::new(path).canonicalize() {
            Err(_) => Err(Reason::NotFound),
            Ok(path) => {
                self.as_pathbuf().clear();
                self.as_pathbuf().push(path);

                Ok(())
            }
        }
    }

    pub fn reset(&self, chunkname: &str) {
        let chunkname = chunkname.strip_suffix(".luau").unwrap();
        let chunkname = chunkname.strip_suffix("/init").unwrap_or(chunkname);

        self.as_pathbuf().clear();
        self.as_pathbuf().push(chunkname);
    }

    pub fn parent(&self) -> Result<(), Reason> {
        if !self.as_pathbuf().pop() {
            Err(Reason::NotFound)
        } else if self.is_ambiguous() {
            Err(Reason::Ambiguous)
        } else {
            Ok(())
        }
    }

    pub fn child(&self, name: &str) -> Result<(), Reason> {
        self.as_pathbuf().push(name);

        if !self.exists() {
            Err(Reason::NotFound)
        } else if self.is_ambiguous() {
            Err(Reason::Ambiguous)
        } else {
            Ok(())
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.as_path().join(".luaurc")
    }

    pub fn config_exists(&self) -> bool {
        self.config_path().is_file()
    }
}

struct Writer {
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
}

impl Writer {
    pub fn new(buffer: *mut c_char, buffer_size: usize, size_out: *mut usize) -> Self {
        Self {
            buffer,
            buffer_size,
            size_out,
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn as_slice(&self, size: usize) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.buffer as *mut u8, size) }
    }

    pub fn set_size(&self, size: usize) -> Result<&mut [u8], ()> {
        unsafe { self.size_out.write(size) };

        if self.buffer_size < size {
            Err(())
        } else {
            Ok(self.as_slice(size))
        }
    }
}

extern "C-unwind" fn is_require_allowed(_: luau::Context, _: Current, _: *const c_char) -> bool {
    true
}

extern "C-unwind" fn reset(
    ctx: luau::Context,
    current: Current,
    chunkname: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    current.reset(ptr_to_str(&ctx, chunkname));

    luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS
}

extern "C-unwind" fn jump_to_alias(
    ctx: luau::Context,
    current: Current,
    path: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    current.jump(ptr_to_str(&ctx, path)).into()
}

extern "C-unwind" fn to_parent(
    _: luau::Context,
    current: Current,
) -> luau::ffi::luarequire_NavigateResult {
    current.parent().into()
}

extern "C-unwind" fn to_child(
    ctx: luau::Context,
    current: Current,
    name: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    current.child(ptr_to_str(&ctx, name)).into()
}

extern "C-unwind" fn is_module_present(_: luau::Context, current: Current) -> bool {
    current.exists()
}

extern "C-unwind" fn get_chunkname(
    _: luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    let writer = Writer::new(buffer, buffer_size, size_out);
    let path = current.as_str().as_bytes();

    let Ok(slice) = writer.set_size(path.len()) else {
        return luau::ffi::luarequire_WriteResult::WRITE_BUFFER_TOO_SMALL;
    };

    slice.copy_from_slice(path);
    luau::ffi::luarequire_WriteResult::WRITE_SUCCESS
}

extern "C-unwind" fn get_loadname(
    _: luau::Context,
    _: Current,
    _: *mut c_char,
    _: usize,
    _: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    luau::ffi::luarequire_WriteResult::WRITE_SUCCESS
}

extern "C-unwind" fn get_cache_key(
    ctx: luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    get_chunkname(ctx, current, buffer, buffer_size, size_out)
}

extern "C-unwind" fn is_config_present(_: &luau::Context, current: Current) -> bool {
    current.config_exists()
}

extern "C-unwind" fn get_config(
    _: luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    let writer = Writer::new(buffer, buffer_size, size_out);
    let path = current.config_path();

    let Ok(mut file) = std::fs::File::open(path) else {
        return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
    };

    let Ok(len) = file.metadata().map(|m| m.len() as usize) else {
        return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
    };

    let Ok(slice) = writer.set_size(len) else {
        return luau::ffi::luarequire_WriteResult::WRITE_BUFFER_TOO_SMALL;
    };

    let Ok(_) = file.read_exact(slice) else {
        return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
    };

    luau::ffi::luarequire_WriteResult::WRITE_SUCCESS
}

fn push_yield_key(stack: &luau::Stack) {
    static mut REG_YIELD_KEY: u8 = b'y';
    stack.push_light_userdata(&raw mut REG_YIELD_KEY as *mut _ as _);
}

fn push_yield_table(stack: &luau::Stack) {
    push_yield_key(stack);
    stack.table_get_raw(luau::REGISTRY_IDX);
}

extern "C-unwind" fn runner(ctx: luau::Context) -> luau::FnReturn {
    // chunkname, userfunc

    push_yield_table(&ctx); // chunkname, userfunc, reqtbl
    ctx.push_copy(1); // chunkname, userfunc, reqtbl, chunkname
    ctx.push_table(); // chunkname, userfunc, reqtbl, chunkname, yldtbl
    ctx.table_set_raw(3); // chunkname, userfunc, reqtbl
    ctx.pop(1); // chunkname, userfunc

    // chunkname, userfunc

    extern "C-unwind" fn handle_error(ctx: luau::Context) -> luau::FnReturn {
        let main = ctx.main();
        main.handle_status(&ctx.thread(), luau::Status::ErrRuntime);

        ctx.ret_with(1)
    }

    ctx.push_copy(2); // chunkname, userfunc, userfunc
    ctx.push_function(c"require_error_handler", handle_error); // chunkname, userfunc, userfunc, errfunc
    ctx.replace(2); // chunkname, errfunc, userfunc

    let success = ctx.pcall(0, 1, 2); // chunkname, errfunc, result
    ctx.remove(-2); // chunkname, result

    if ctx.thread().status() == luau::Status::Yield {
        -1 // special yield indicator
    } else {
        runner_cont(ctx, success)
    }
}

extern "C-unwind" fn runner_cont(ctx: luau::Context, status: luau::Status) -> luau::FnReturn {
    // chunkname, result

    let main = ctx.main();

    push_yield_table(&ctx); // chunkname, result, reqtbl
    ctx.push_copy(1); // chunkname, result, reqtbl, chunkname
    ctx.table_get_raw(3); // chunkname, result, reqtbl, yldtbl

    for i in 1..ctx.len(-1) {
        ctx.table_get_raw_i(-1, i as u32); // chunkname, result, reqyld, yldtbl, thread
        let thread = ctx.to_thread(-1).unwrap();
        ctx.pop(1); // chunkname, result, reqyld, yldtbl

        if status == luau::Status::Ok {
            ctx.xpush(&thread, 2);
            main.spawn(&thread, 1);
        } else {
            thread
                .stack()
                .push_string("required module errored while loading");
            main.spawn_error(&thread)
        }
    }

    ctx.pop(1); // chunkname, result, reqyld

    ctx.push_copy(1); // chunkname, result, reqyld, chunkname
    ctx.push_nil(); // chunkname, result, reqyld, chunkname, nil
    ctx.table_set_raw(3); // chunkname, result, reqyld

    ctx.pop(1); // chunkname, result

    if status == luau::Status::Ok {
        ctx.ret_with(1)
    } else {
        ctx.error()
    }
}

extern "C-unwind" fn load(
    ctx: luau::Context,
    current: Current,
    _: *const c_char,
    chunkname: *const c_char,
    _: *const c_char,
) -> c_int {
    let chunkname = unsafe { CStr::from_ptr(chunkname) };

    push_yield_table(&ctx); // reqtbl
    ctx.push_string(chunkname.to_bytes()); // reqtbl, chunkname
    ctx.table_get_raw(-2); // reqtbl, yldtbl

    if !ctx.is_nil(-1) {
        ctx.push_thread(&ctx.thread()); // reqtbl, yldtbl, thread
        ctx.table_set_raw_i(-2, ctx.len(-2) as u32 + 1); // reqtbl, yldtbl

        return -1; // special yield indicator
    }

    ctx.pop(2); // stack is empty

    let source = {
        let (first, second) = current.possible_paths();

        if first.exists() {
            std::fs::read(&first).unwrap_or_else(|_| {
                ctx.push_error(format!("failed to read file '{}'", first.display()))
            })
        } else {
            std::fs::read(&second).unwrap_or_else(|_| {
                ctx.push_error(format!("failed to read file '{}'", second.display()))
            })
        }
    };

    let main = ctx.main();
    let (_, thread) = main.new_thread();
    let stack = thread.stack();

    stack.push_function_cont(c"require_runner", runner, runner_cont);
    stack.push_string(chunkname.to_bytes());
    stack.push_bytecode(chunkname, &main.compiler().compile(&source));

    match thread.resume(None, 2) {
        luau::Status::Ok => {
            stack.xpush(&ctx.thread(), -1);

            0
        }

        luau::Status::Yield => {
            push_yield_table(&ctx); // reqtbl
            ctx.push_string(chunkname.to_bytes()); // reqtbl, chunkname
            ctx.table_get_raw(-2); // reqtbl, yldtbl
            ctx.push_thread(&ctx.thread()); // reqtbl, yldtbl, thread
            ctx.table_set_raw_i(-2, ctx.len(-2) as u32 + 1); // reqtbl, yldtbl

            -1
        }

        _ => ctx.push_error("required module errored while loading"),
    }
}

pub fn open(main: luau::Main) {
    #[allow(clippy::missing_transmute_annotations)]
    extern "C-unwind" fn luarequire_configuration_init(
        config: *mut luau::ffi::luarequire_Configuration,
    ) {
        use std::mem::transmute;

        unsafe {
            (*config).is_require_allowed = transmute(is_require_allowed as *mut c_void);
            (*config).reset = transmute(reset as *mut c_void);
            (*config).jump_to_alias = transmute(jump_to_alias as *mut c_void);
            (*config).to_parent = transmute(to_parent as *mut c_void);
            (*config).to_child = transmute(to_child as *mut c_void);
            (*config).is_module_present = transmute(is_module_present as *mut c_void);
            (*config).get_chunkname = transmute(get_chunkname as *mut c_void);
            (*config).get_loadname = transmute(get_loadname as *mut c_void);
            (*config).get_cache_key = transmute(get_cache_key as *mut c_void);
            (*config).is_config_present = transmute(is_config_present as *mut c_void);
            (*config).get_config = transmute(get_config as *mut c_void);
            (*config).load = transmute(load as *mut c_void);
        }
    }

    let stack = main.stack();

    push_yield_key(&stack);
    stack.push_table();
    stack.table_set_raw(luau::REGISTRY_IDX);

    unsafe {
        luau::ffi::luaopen_require(
            main.as_ptr(),
            luarequire_configuration_init,
            Box::into_raw(Box::new(PathBuf::new())) as *mut c_void,
        );
    }
}
