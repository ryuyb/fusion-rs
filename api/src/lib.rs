use crate::config::Config;
use crate::job::JobManager;
use crate::notification::bark::BarkProvider;
use crate::notification::{NotificationCenter, NotificationProvider};
use crate::utils::jwt::JwtUtil;
use live_platform::LivePlatformProvider;
use std::sync::Arc;
use tokio::signal;

mod api;
mod config;
mod domain;
mod dto;
mod error;
mod job;
mod notification;
mod repository;
mod service;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub services: Arc<service::Services>,
    pub jwt: Arc<JwtUtil>,
    pub live_platform_provider: Arc<LivePlatformProvider>,
    pub notification_center: Arc<NotificationCenter>,
}

pub struct Application {
    pub config: Config,
    pub router: axum::Router,
    pub job_manager: JobManager,
    pub state: Arc<AppState>,
}

impl Application {
    pub async fn build(app_config: Config) -> anyhow::Result<Self> {
        let db = app_config.database.create_db().await?;

        let repos = Arc::new(repository::Repositories::new(db));

        let jwt = Arc::new(JwtUtil::new(app_config.jwt.clone()));
        let services = Arc::new(service::Services::build(repos, jwt.clone()).await?);

        let live_platform_provider = Arc::new(LivePlatformProvider::new()?);

        let bark_provider: Arc<dyn NotificationProvider> = Arc::new(BarkProvider::new()?);
        let notification_center = Arc::new(NotificationCenter::with_providers(vec![bark_provider]));

        let state = Arc::new(AppState {
            config: app_config.clone(),
            services,
            jwt,
            live_platform_provider,
            notification_center,
        });

        let router = api::routes::create_router(state.clone());

        let job_manager = JobManager::new(state.clone()).await?;

        Ok(Self {
            config: app_config.clone(),
            router,
            job_manager,
            state,
        })
    }

    //noinspection HttpUrlsUsage
    pub async fn run(mut self) -> anyhow::Result<()> {
        self.start_job().await?;

        let listener = tokio::net::TcpListener::bind(&self.config.server_address()).await?;
        tracing::info!("Listening on http://{}", self.config.server_address());
        tracing::info!(
            "Swagger docs on http://{}/swagger-ui",
            self.config.server_address()
        );
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        self.job_manager.shutdown().await?;

        Ok(())
    }

    pub async fn start_job(&self) -> anyhow::Result<()> {
        for (name, cfg) in self.config.jobs.iter() {
            if !cfg.enabled {
                continue;
            }
            match self.job_manager.registry().get(name) {
                Some(job) => {
                    self.job_manager
                        .add_job(cfg.cron_expr.as_str(), job)
                        .await?;
                    tracing::info!("Adding job {}", name);
                }
                None => anyhow::bail!("Job {} not available", name),
            };
        }

        self.job_manager.start().await?;
        tracing::info!("Job manager started");
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
