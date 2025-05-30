use std::path::Path;

use crate::{library, luau, runtime};

pub struct Fs;
library!(Fs, read, write);

impl Fs {
    extern "C-unwind" fn read(ctx: luau::Context) -> luau::FnReturn {
        let path = Path::new(ctx.arg_string_str(1)).to_owned();
        let main = ctx.main();
        let r = ctx.thread().to_ref();
        ctx.spawner().spawn(async move {
            let result = runtime::fs::read(path).await;
            let thread = r.to_thread();

            match result {
                Ok(data) => {
                    thread.stack().push_string(data);
                    main.spawn(&thread, 1);
                }

                Err(e) => {
                    thread.stack().push_string(e.to_string());
                    main.spawn_error(&thread);
                }
            }

            drop(r);
        });

        ctx.yld()
    }

    extern "C-unwind" fn write(ctx: luau::Context) -> luau::FnReturn {
        let path = Path::new(ctx.arg_string_str(1)).to_owned();
        let data = match ctx.type_of(2) {
            luau::Type::String => ctx.to_string_slice(2).unwrap().to_vec(),
            luau::Type::Buffer => ctx.to_buffer(2).unwrap().to_vec(),

            _ => ctx.push_error("bad argument #2 to 'write' (string or buffer expected)"),
        };

        let main = ctx.main();
        let r = ctx.thread().to_ref();
        ctx.spawner().spawn(async move {
            let result = runtime::fs::write(path, data).await;
            let thread = r.to_thread();

            match result {
                Ok(_) => {
                    thread.stack().push_nil();
                    main.spawn(&thread, 1);
                }

                Err(e) => {
                    thread.stack().push_string(e.to_string());
                    main.spawn_error(&thread);
                }
            }

            drop(r);
        });

        ctx.yld()
    }
}
