use std::time::Duration;

use crate::{luau, runtime};

pub fn open(main: &luau::Main) {
    extern "C-unwind" fn spawn(ctx: luau::Context) -> luau::FnReturn {
        let thread = match ctx.type_of(1) {
            luau::Type::Thread => ctx.to_thread(1).unwrap(),
            luau::Type::Function => {
                let thread = ctx.push_thread_new();

                ctx.xpush(&thread, 1);
                ctx.replace(1);

                thread
            }

            _ => ctx.push_error("bad argument #1 to function (thread or function expected)"),
        };

        let nargs = ctx.get_top() - 1;
        ctx.xmove(&thread, nargs);

        ctx.main().spawn(&thread, nargs);

        ctx.ret_with(1)
    }

    extern "C-unwind" fn defer(ctx: luau::Context) -> luau::FnReturn {
        let thread = match ctx.type_of(1) {
            luau::Type::Thread => ctx.to_thread(1).unwrap(),
            luau::Type::Function => {
                let thread = ctx.push_thread_new();

                ctx.xpush(&thread, 1);
                ctx.replace(1);

                thread
            }

            _ => ctx.push_error("bad argument #1 to function (thread or function expected)"),
        };

        let r = ctx.to_ref(1);

        let nargs = ctx.get_top() - 1;
        ctx.xmove(&thread, nargs);

        let main = ctx.main();
        ctx.spawner().defer(async move {
            main.spawn(&thread, nargs);
            drop(r);
        });

        ctx.ret_with(1)
    }

    extern "C-unwind" fn delay(ctx: luau::Context) -> luau::FnReturn {
        let thread = match ctx.type_of(1) {
            luau::Type::Thread => ctx.to_thread(1).unwrap(),
            luau::Type::Function => {
                let thread = ctx.push_thread_new();

                ctx.xpush(&thread, 1);
                ctx.replace(1);

                thread
            }

            _ => ctx.push_error("bad argument #1 to function (thread or function expected)"),
        };

        let r = ctx.to_ref(1);

        let delay = Duration::from_secs_f64(ctx.arg_number(2));

        let nargs = ctx.get_top() - 2;
        ctx.xmove(&thread, nargs);

        let main = ctx.main();
        ctx.spawner().spawn(async move {
            runtime::sleep(delay).await;
            main.spawn(&thread, nargs);
            drop(r);
        });

        ctx.pop(1);
        ctx.ret_with(1)
    }

    extern "C-unwind" fn wait(ctx: luau::Context) -> luau::FnReturn {
        let delay = Duration::from_secs_f64(ctx.arg_number(1));

        let thread = ctx.thread();

        ctx.push_thread(&thread);
        let r = ctx.to_ref(-1);
        ctx.pop(1);

        let main = ctx.main();
        ctx.spawner().spawn(async move {
            runtime::sleep(delay).await;
            main.spawn(&thread, 0);
            drop(r);
        });

        ctx.yld()
    }

    let stack = main.stack();
    stack.push_table();

    stack.push_function(c"task.spawn", spawn);
    stack.table_set_field(-2, c"spawn");

    stack.push_function(c"task.defer", defer);
    stack.table_set_field(-2, c"defer");

    stack.push_function(c"task.delay", delay);
    stack.table_set_field(-2, c"delay");

    stack.push_function(c"task.wait", wait);
    stack.table_set_field(-2, c"wait");
}
