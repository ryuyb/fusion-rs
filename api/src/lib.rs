use crate::config::Config;
use crate::utils::jwt::JwtUtil;
use std::sync::Arc;
use tokio::signal;

mod api;
mod config;
mod domain;
mod dto;
mod error;
mod repository;
mod service;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub services: Arc<service::Services>,
    pub jwt: Arc<JwtUtil>,
}

pub struct Application {
    pub config: Config,
    pub router: axum::Router,
}

impl Application {
    pub async fn build(app_config: Config) -> anyhow::Result<Self> {
        let db = app_config.database.create_db().await?;

        let repos = Arc::new(repository::Repositories::new(db));

        let jwt = Arc::new(JwtUtil::new(app_config.jwt.clone()));
        let services = Arc::new(service::Services::build(repos, jwt.clone()).await?);

        let state = Arc::new(AppState {
            config: app_config.clone(),
            services,
            jwt,
        });

        let router = api::routes::create_router(state);

        Ok(Self {
            config: app_config.clone(),
            router,
        })
    }

    //noinspection HttpUrlsUsage
    pub async fn run(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.config.server_address()).await?;
        tracing::info!("Listening on http://{}", self.config.server_address());
        tracing::info!(
            "Swagger docs on http://{}/swagger-ui",
            self.config.server_address()
        );
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn main() {
    let app_config = config::load().expect("Failed to load configuration");

    let _logging_guard = app_config
        .logging
        .init_subscriber()
        .expect("Failed to initialize logging");
    tracing_log::LogTracer::init().expect("Failed to set logger");

    tracing::info!(
        "Configuration loaded for environment: {}",
        app_config.environment.as_str()
    );

    let app = match Application::build(app_config).await {
        Ok(app) => app,
        Err(error) => panic!("Failed to build application{:?}", error),
    };
    app.run().await.expect("Failed to run application");
}
