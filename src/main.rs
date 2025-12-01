use crate::cli::{Cli, Commands, MigrateCommands};
use clap::Parser;

mod cli;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Serve => api::main(),
        Commands::Migrate(migrate_args) => match migrate_args.direction {
            MigrateCommands::Up => println!("up"),
            MigrateCommands::Down => println!("down"),
        },
    }
}
