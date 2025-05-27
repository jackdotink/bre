use std::{
    ffi::{CStr, c_char, c_int, c_void},
    io::Read,
    path::{Path, PathBuf},
};

use crate::luau;

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
}

fn is_ambiguous(current: Current) -> bool {
    let path = current.as_path();

    path.is_dir() && path.with_extension("luau").is_file()
}

fn cptr_to_str(ctx: &luau::Context, ptr: *const c_char) -> &str {
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .unwrap_or_else(|_| ctx.push_error("non-utf8 string"))
    }
}

fn path_to_str<'path>(ctx: &luau::Context, path: &'path Path) -> &'path str {
    path.as_os_str()
        .to_str()
        .unwrap_or_else(|| ctx.push_error("non-utf8 path"))
}

extern "C-unwind" fn is_require_allowed(_: luau::Context, _: Current, _: *const c_char) -> bool {
    true
}

extern "C-unwind" fn reset(
    ctx: luau::Context,
    current: Current,
    chunkname: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    let chunkname = cptr_to_str(&ctx, chunkname);
    let chunkname = chunkname.strip_suffix(".luau").unwrap();
    let chunkname = chunkname.strip_suffix("/init").unwrap_or(chunkname);

    current.as_pathbuf().push(chunkname);

    luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS
}

extern "C-unwind" fn jump_to_alias(
    ctx: luau::Context,
    current: Current,
    path: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    let path = Path::new(cptr_to_str(&ctx, path));

    match path.canonicalize() {
        Ok(path) => current.as_pathbuf().push(path),
        Err(_) => return luau::ffi::luarequire_NavigateResult::NAVIGATE_NOT_FOUND,
    }

    if is_ambiguous(current) {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_AMBIGUOUS
    } else {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS
    }
}

extern "C-unwind" fn to_parent(
    _: luau::Context,
    current: Current,
) -> luau::ffi::luarequire_NavigateResult {
    if !current.as_pathbuf().pop() {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_NOT_FOUND
    } else if is_ambiguous(current) {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_AMBIGUOUS
    } else {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS
    }
}

extern "C-unwind" fn to_child(
    ctx: luau::Context,
    current: Current,
    name: *const c_char,
) -> luau::ffi::luarequire_NavigateResult {
    let name = Path::new(cptr_to_str(&ctx, name));
    current.as_pathbuf().push(name);

    if !(current.as_path().is_dir() || current.as_path().with_extension("luau").is_file()) {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_NOT_FOUND
    } else if is_ambiguous(current) {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_AMBIGUOUS
    } else {
        luau::ffi::luarequire_NavigateResult::NAVIGATE_SUCCESS
    }
}

extern "C-unwind" fn is_module_present(_: luau::Context, current: Current) -> bool {
    let path = current.as_path();

    path.with_extension("luau").is_file() || path.join("./init.luau").is_file()
}

extern "C-unwind" fn get_chunkname(
    ctx: luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    let path = path_to_str(&ctx, current.as_path());
    let len = path.len();

    unsafe { size_out.write(len) };
    if buffer_size < len {
        luau::ffi::luarequire_WriteResult::WRITE_BUFFER_TOO_SMALL
    } else {
        let buffer = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, len) };
        buffer.copy_from_slice(path.as_bytes());

        luau::ffi::luarequire_WriteResult::WRITE_SUCCESS
    }
}

extern "C-unwind" fn get_loadname(
    ctx: &luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
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

extern "C-unwind" fn is_config_present(ctx: &luau::Context, current: Current) -> bool {
    current.as_path().join(".luaurc").is_file()
}

extern "C-unwind" fn get_config(
    ctx: &luau::Context,
    current: Current,
    buffer: *mut c_char,
    buffer_size: usize,
    size_out: *mut usize,
) -> luau::ffi::luarequire_WriteResult {
    let path = current.as_path().join(".luaurc");
    let Ok(mut file) = std::fs::File::open(path) else {
        return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
    };

    let Ok(len) = file.metadata().map(|m| m.len() as usize) else {
        return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
    };

    unsafe { size_out.write(len) };
    if buffer_size < len {
        luau::ffi::luarequire_WriteResult::WRITE_BUFFER_TOO_SMALL
    } else {
        let buffer = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, len) };
        let Ok(_) = file.read_exact(buffer) else {
            return luau::ffi::luarequire_WriteResult::WRITE_FAILURE;
        };

        luau::ffi::luarequire_WriteResult::WRITE_SUCCESS
    }
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

    push_yield_table(&ctx);
    ctx.push_copy(-3);
    ctx.push_table();
    ctx.table_set_raw(-3);
    ctx.pop(1);

    // chunkname, userfunc

    extern "C-unwind" fn handle_error(ctx: luau::Context) -> luau::FnReturn {
        let main = ctx.main();
        main.handle_status(&ctx.thread(), luau::Status::ErrRuntime);

        ctx.ret_with(1)
    }

    ctx.push_copy(-1);
    // chunkname, userfunc, userfunc

    ctx.push_function(c"require_error_handler", handle_error);
    // chunkname, userfunc, userfunc, errfunc

    ctx.replace(-3);
    // chunkname, errfunc, userfunc

    let success = ctx.pcall(0, 1, -2);
    // chunkname, errfunc, result

    ctx.remove(-2);
    // chunkname, result

    if ctx.thread().status() == luau::Status::Yield {
        -1 // special yield indicator
    } else {
        runner_cont(ctx, success)
    }
}

extern "C-unwind" fn runner_cont(ctx: luau::Context, status: luau::Status) -> luau::FnReturn {
    // chunkname, result

    let main = ctx.main();

    push_yield_table(&ctx);
    ctx.push_copy(-2);
    ctx.table_get_raw(-2);

    // chunkname, result, reqyld, yldtbl

    for i in 1..ctx.len(-1) {
        ctx.table_get_raw_i(-1, i as u32);
        // chunkname, result, reqyld, yldtbl, thread

        let thread = ctx.to_thread(-1).unwrap();

        ctx.pop(1);
        // chunkname, result, reqyld, yldtbl

        if status == luau::Status::Ok {
            ctx.xpush(&thread, -3);
            main.spawn(&thread, 1);
        } else {
            thread
                .stack()
                .push_string("required module errored while loading");
            main.spawn_error(&thread)
        }
    }

    // chunkname, result, reqyld, yldtbl
    ctx.pop(1);

    ctx.push_copy(-3);
    ctx.push_nil();
    ctx.table_set_raw(-3);

    ctx.pop(1);

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
    chunkname_ptr: *const c_char,
    _: *const c_char,
) -> c_int {
    let Ok(source) = std::fs::read(current.as_path().with_extension("luau")) else {
        ctx.push_error(format!(
            "failed to read file '{}'",
            current.as_path().display()
        ));
    };

    let chunkname = cptr_to_str(&ctx, chunkname_ptr);

    let main = ctx.main();

    let (_, thread) = main.new_thread();
    let stack = thread.stack();

    stack.push_function_cont(c"require_runner", runner, runner_cont);
    stack.push_string(chunkname);
    stack.push_bytecode(
        unsafe { CStr::from_ptr(chunkname_ptr) },
        &main.compiler().compile(&source),
    );

    match thread.resume(None, 2) {
        luau::Status::Ok => {
            stack.xpush(&ctx.thread(), -1);

            0
        }

        luau::Status::Yield => {
            push_yield_table(&ctx);
            ctx.push_string(chunkname);
            ctx.table_get_raw(-2);

            ctx.push_thread(&ctx.thread());
            ctx.table_set_raw_i(-2, ctx.len(-2) as u32 + 1);

            -1
        }

        _ => ctx.push_error("required module errored while loading"),
    }
}

pub fn open(main: &luau::Main) {
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
