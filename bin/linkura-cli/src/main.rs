use clap::Parser;

use config::init;

use linkura_common::log;

rust_i18n::i18n!("locales", fallback = "en");

mod command;
mod config;
mod cli;

use rust_i18n::t;

use crate::config::Commands;
fn main() {
    let args = config::Args::parse();
    if !args.quiet {
        log::init(args.log_level.clone());
    }

    let global = init(args).expect(&t!("config.initialize.failed"));
    let args = &global.args;

    match &args.command {
        Some(Commands::MRS(_)) => {
            todo!("Implement MRS client command handling");
        }
        Some(Commands::ALS(args)) => {
            let _ = command::als::run(
                &global,
                command::als::AlsConnectionInfo {
                    address: args.addr.clone(),
                    port: args.port,
                    room_id: args.room_id.clone(),
                    token: args.token.clone(),
                },
                args.watch,
            )
            .map_err(|e| {
                tracing::error!("Error running ALS command: {}", e);
                std::process::exit(1);
            });
        }
        Some(Commands::Archive(args)) => {
            let _ = command::archive::run(&global, &args).map_err(|e| {
                tracing::error!("Error running Archive command: {}", e);
                std::process::exit(1);
            });
        }
        None => {
            command::default::run(&global);
        }
    }

    // If user sets --stay-open, wait for an Enter key before exiting
    if global.args.stay_open {
        use std::io::{self, Write, BufRead};
        println!("Press Enter to exit...");
        let _ = io::stdout().flush();
        let mut _input = String::new();
        let _ = io::stdin().read_line(&mut _input);
    }

    return;
}
