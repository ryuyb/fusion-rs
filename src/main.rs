use crate::cli::{Cli, Commands, MigrateCommands};
use clap::Parser;
use std::env;

mod cli;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Serve(serve_args) => {
            if let Some(port) = serve_args.port {
                // SAFETY: setting process-level environment variable for the current process
                // before the async runtime starts; acceptable for overriding config.
                unsafe {
                    env::set_var("FUSION_SERVER_PORT", port.to_string());
                }
            }
            api::main()
        }
        Commands::Migrate(migrate_args) => match migrate_args.direction {
            MigrateCommands::Up => println!("up"),
            MigrateCommands::Down => println!("down"),
        },
    }
}
