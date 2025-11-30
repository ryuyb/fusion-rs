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
    Serve,

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
}
