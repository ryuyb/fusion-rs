use crate::cli::{Cli, Commands, MigrateCommands};
use clap::Parser;
use migration::Migrator;
use migration::sea_orm::{ConnectOptions, Database};
use std::env;

mod cli;

const BUILD_GIT_TAG: &str = env!("FUSION_BUILD_GIT_TAG");
const BUILD_GIT_COMMIT: &str = env!("FUSION_BUILD_GIT_COMMIT");
const BUILD_TIMESTAMP: &str = env!("FUSION_BUILD_TIMESTAMP");
const BUILD_RUST_VERSION: &str = env!("FUSION_BUILD_RUST_VERSION");

#[tokio::main]
async fn main() {
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
            api::main().await
        }
        Commands::Migrate(migrate_args) => {
            let database_url = env::var("FUSION_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .expect("FUSION_DATABASE_URL or DATABASE_URL must be set");
            let opt = ConnectOptions::new(database_url).to_owned();
            let db = Database::connect(opt)
                .await
                .expect("Database connection failed.");
            use migration::MigratorTrait;
            match migrate_args.direction {
                MigrateCommands::Up => {
                    Migrator::up(&db, None)
                        .await
                        .expect("Failed to run migrate up");
                    println!("Successfully migrated.");
                }
                MigrateCommands::Down => {
                    Migrator::down(&db, None)
                        .await
                        .expect("Failed to run migrate down");
                    println!("Successfully migrated.");
                }
                MigrateCommands::Version => {
                    let vec = Migrator::get_migration_with_status(&db)
                        .await
                        .expect("Failed to run migrate down");
                    println!(
                        "Current version: {}, status: {}",
                        vec[0].name(),
                        vec[0].status()
                    );
                }
            }
        }
        Commands::Version => {
            println!("version        : {}", BUILD_GIT_TAG);
            println!("git commit     : {}", BUILD_GIT_COMMIT);
            println!("build time     : {}", BUILD_TIMESTAMP);
            println!("rustc version  : {}", BUILD_RUST_VERSION);
        }
    }
}
