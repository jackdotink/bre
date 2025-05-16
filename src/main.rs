mod luau;
mod runtime;

mod task;

fn main() {
    let compiler = luau::Compiler::default();
    let bytecode = compiler.compile(std::fs::read_to_string("main.luau").unwrap().as_str());

    let executor = runtime::Executor::default();
    let luau = luau::Luau::new(executor.spawner());

    luau.execute(c"main", &bytecode);
    executor.run();
}
