use std::time::Duration;

use crate::{library, luau, runtime};

pub struct Task;
library!(Task, spawn, defer, delay, wait);

impl Task {
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

        {
            let main = ctx.main();
            ctx.spawner().defer(async move {
                main.spawn(&r.to_thread(), nargs);

                drop(r);
            });
        }

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

        {
            let main = ctx.main();
            ctx.spawner().spawn(async move {
                runtime::time::sleep(delay).await;
                main.spawn(&r.to_thread(), nargs);

                drop(r);
            });
        }

        ctx.pop(1);
        ctx.ret_with(1)
    }

    extern "C-unwind" fn wait(ctx: luau::Context) -> luau::FnReturn {
        let delay = Duration::from_secs_f64(ctx.arg_number(1));

        let thread = ctx.thread();

        ctx.push_thread(&thread);
        let r = ctx.to_ref(-1);
        ctx.pop(1);

        {
            let main = ctx.main();
            ctx.spawner().spawn(async move {
                runtime::time::sleep(delay).await;
                main.spawn(&r.to_thread(), 0);

                drop(r);
            });
        }

        ctx.yld()
    }
}
