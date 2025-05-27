use std::{fs::read, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::{luau, runtime};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The command to execute.
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the project in the current directory.
    Run {
        /// The optimization level to compile with.
        #[arg(short, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=2))]
        opt_level: u8,
    },
}

pub fn cli() {
    let args = Args::parse();

    match args.command {
        Commands::Run { opt_level } => {
            let executor = runtime::Executor::default();
            let compiler = luau::Compiler::default().with_opt_level(opt_level.try_into().unwrap());
            let luau = luau::Luau::new(executor.spawner(), compiler.clone());

            let path = PathBuf::from("./main.luau").canonicalize().unwrap();
            let code = std::fs::read(&path).unwrap();

            let bytecode = compiler.compile(&code);
            luau.execute(&path, &bytecode);

            executor.run();
        }
    }
}
