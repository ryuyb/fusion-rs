use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "fusion")]
#[command(about = "A fusion CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Start the API server")]
    Serve(ServeArgs),

    #[command(arg_required_else_help = true, about = "Run database migrations")]
    Migrate(MigrateArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct MigrateArgs {
    #[command(subcommand)]
    pub direction: MigrateCommands,
}

#[derive(Debug, Subcommand)]
pub enum MigrateCommands {
    #[command(about = "Run migrations up")]
    Up,
    #[command(about = "Rollback migrations")]
    Down,
    #[command(about = "Show migration version")]
    Version,
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, short, help = "Override server port from config", value_parser = clap::value_parser!(u16).range(1..=65535))]
    pub port: Option<u16>,
}
