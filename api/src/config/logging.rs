use anyhow::Context;
use chrono::Utc;
use flate2::{Compression, write::GzEncoder};
use serde::Deserialize;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tracing::{Dispatch, level_filters::LevelFilter, subscriber};
use tracing_appender::non_blocking::{self, NonBlocking, WorkerGuard};
use tracing_core::{Event, Interest, Metadata, span};
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::MakeWriter,
    layer::{Context as LayerContext, SubscriberExt},
};

type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: LogLevel,

    #[serde(default = "default_true")]
    pub console_enabled: bool,

    #[serde(default = "default_true")]
    pub console_color: bool,

    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    #[serde(default)]
    pub file: Option<LogFileConfig>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            console_enabled: default_true(),
            console_color: default_true(),
            format: default_log_format(),
            file: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Full,
    Compact,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogFileConfig {
    pub path: String,

    #[serde(default = "default_true")]
    pub create_parent_dirs: bool,

    #[serde(default = "default_true")]
    pub append: bool,

    #[serde(default)]
    pub rotation: Option<LogRotationConfig>,

    #[serde(default)]
    pub format: Option<LogFormat>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum LogRotationConfig {
    Size {
        #[serde(default = "default_log_rotation_max_size_bytes")]
        max_size_bytes: u64,
        #[serde(default = "default_log_rotation_max_files")]
        max_files: usize,
        #[serde(default = "default_log_rotation_compress")]
        compress: bool,
    },
    Interval {
        #[serde(default = "default_log_rotation_interval_hours")]
        interval_hours: u64,
        #[serde(default = "default_log_rotation_max_files")]
        max_files: usize,
        #[serde(default = "default_log_rotation_compress")]
        compress: bool,
    },
}

#[derive(Debug)]
pub struct LoggingGuard {
    _file_guard: Option<WorkerGuard>,
}

impl LoggingGuard {
    fn new(file_guard: Option<WorkerGuard>) -> Self {
        Self {
            _file_guard: file_guard,
        }
    }
}

struct LayerStack {
    layers: Vec<BoxedLayer>,
}

impl LayerStack {
    fn new(layers: Vec<BoxedLayer>) -> Self {
        Self { layers }
    }
}

impl Layer<Registry> for LayerStack {
    fn on_register_dispatch(&self, dispatch: &Dispatch) {
        for layer in &self.layers {
            layer.on_register_dispatch(dispatch);
        }
    }

    fn on_layer(&mut self, subscriber: &mut Registry) {
        for layer in &mut self.layers {
            layer.on_layer(subscriber);
        }
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        let mut interest = Interest::never();
        for layer in &self.layers {
            let inner = layer.register_callsite(metadata);
            if inner.is_always() {
                return inner;
            }
            if inner.is_sometimes() {
                interest = Interest::sometimes();
            }
        }
        interest
    }

    fn enabled(&self, metadata: &Metadata<'_>, ctx: LayerContext<'_, Registry>) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.enabled(metadata, ctx.clone()))
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        self.layers
            .iter()
            .filter_map(|layer| layer.max_level_hint())
            .max()
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: LayerContext<'_, Registry>,
    ) {
        for layer in &self.layers {
            layer.on_new_span(attrs, id, ctx.clone());
        }
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        ctx: LayerContext<'_, Registry>,
    ) {
        for layer in &self.layers {
            layer.on_record(span, values, ctx.clone());
        }
    }

    fn on_follows_from(
        &self,
        span: &span::Id,
        follows: &span::Id,
        ctx: LayerContext<'_, Registry>,
    ) {
        for layer in &self.layers {
            layer.on_follows_from(span, follows, ctx.clone());
        }
    }

    fn event_enabled(&self, event: &Event<'_>, ctx: LayerContext<'_, Registry>) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.event_enabled(event, ctx.clone()))
    }

    fn on_event(&self, event: &Event<'_>, ctx: LayerContext<'_, Registry>) {
        for layer in &self.layers {
            layer.on_event(event, ctx.clone());
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: LayerContext<'_, Registry>) {
        for layer in &self.layers {
            layer.on_enter(id, ctx.clone());
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: LayerContext<'_, Registry>) {
        for layer in &self.layers {
            layer.on_exit(id, ctx.clone());
        }
    }

    fn on_close(&self, id: span::Id, ctx: LayerContext<'_, Registry>) {
        for layer in &self.layers {
            layer.on_close(id.clone(), ctx.clone());
        }
    }

    fn on_id_change(&self, old: &span::Id, new: &span::Id, ctx: LayerContext<'_, Registry>) {
        for layer in &self.layers {
            layer.on_id_change(old, new, ctx.clone());
        }
    }
}
impl LoggingConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(file) = &self.file {
            if file.path.trim().is_empty() {
                anyhow::bail!("logging.file.path must not be empty when file output is enabled");
            }

            if let Some(rotation) = &file.rotation {
                rotation.validate()?;
            }
        }

        Ok(())
    }

    pub fn init_subscriber(&self) -> anyhow::Result<LoggingGuard> {
        let level_filter: LevelFilter = self.level.into();
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level_filter.to_string()));

        let mut layers: Vec<BoxedLayer> = Vec::new();
        let mut file_guard = None;

        if let Some(file_config) = &self.file {
            let (writer, guard) = file_config.build_writer()?;
            let format = file_config.format.unwrap_or(self.format);
            layers.push(build_layer(
                format,
                writer,
                false,
                level_filter,
                env_filter.clone(),
            ));
            file_guard = Some(guard);
        }

        if self.console_enabled || layers.is_empty() {
            layers.push(build_layer(
                self.format,
                || io::stdout(),
                self.console_color,
                level_filter,
                env_filter.clone(),
            ));
        }

        let stack = LayerStack::new(layers);

        let subscriber = tracing_subscriber::registry().with(stack);

        subscriber::set_global_default(subscriber)
            .context("Failed to install tracing subscriber")?;

        Ok(LoggingGuard::new(file_guard))
    }
}

impl LogRotationConfig {
    fn validate(&self) -> anyhow::Result<()> {
        match self {
            LogRotationConfig::Size {
                max_size_bytes,
                max_files,
                ..
            } => {
                if *max_size_bytes == 0 {
                    anyhow::bail!("logging.file.rotation.max_size_bytes must be greater than zero");
                }
                if *max_files == 0 {
                    anyhow::bail!("logging.file.rotation.max_files must be greater than zero");
                }
            }
            LogRotationConfig::Interval {
                interval_hours,
                max_files,
                ..
            } => {
                if *interval_hours == 0 {
                    anyhow::bail!("logging.file.rotation.interval_hours must be greater than zero");
                }
                if *max_files == 0 {
                    anyhow::bail!("logging.file.rotation.max_files must be greater than zero");
                }
            }
        }

        Ok(())
    }
}

impl LogFileConfig {
    fn build_writer(&self) -> anyhow::Result<(NonBlocking, WorkerGuard)> {
        let path = PathBuf::from(&self.path);
        if self.create_parent_dirs {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create log directory {}", parent.display())
                    })?;
                }
            }
        }

        let strategy = self
            .rotation
            .as_ref()
            .map(FileRotationStrategy::from)
            .unwrap_or(FileRotationStrategy::None);

        let writer = RotatingFileWriter::new(path, self.append, strategy)
            .with_context(|| format!("Failed to initialize log file {}", self.path))?;
        let (non_blocking_writer, guard) = non_blocking::NonBlocking::new(writer);
        Ok((non_blocking_writer, guard))
    }
}

fn build_layer<W>(
    format: LogFormat,
    writer: W,
    ansi: bool,
    level: LevelFilter,
    env_filter: EnvFilter,
) -> BoxedLayer
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    match format {
        LogFormat::Full => tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .with_ansi(ansi)
            .with_filter(env_filter)
            .with_filter(level)
            .boxed(),
        LogFormat::Compact => tracing_subscriber::fmt::layer()
            .compact()
            .with_writer(writer)
            .with_ansi(ansi)
            .with_filter(env_filter)
            .with_filter(level)
            .boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_ansi(ansi)
            .with_filter(env_filter)
            .with_filter(level)
            .boxed(),
    }
}

#[derive(Clone)]
enum FileRotationStrategy {
    None,
    Size(SizeRotation),
    Interval(IntervalRotation),
}

#[derive(Clone)]
struct SizeRotation {
    max_size_bytes: u64,
    max_files: Option<usize>,
    compress: bool,
}

#[derive(Clone)]
struct IntervalRotation {
    interval: Duration,
    max_files: Option<usize>,
    compress: bool,
}

impl From<&LogRotationConfig> for FileRotationStrategy {
    fn from(value: &LogRotationConfig) -> Self {
        match value {
            LogRotationConfig::Size {
                max_size_bytes,
                max_files,
                compress,
            } => FileRotationStrategy::Size(SizeRotation {
                max_size_bytes: *max_size_bytes,
                max_files: Some(*max_files),
                compress: *compress,
            }),
            LogRotationConfig::Interval {
                interval_hours,
                max_files,
                compress,
            } => FileRotationStrategy::Interval(IntervalRotation {
                interval: Duration::from_secs(interval_hours.saturating_mul(3600)),
                max_files: Some(*max_files),
                compress: *compress,
            }),
        }
    }
}

impl FileRotationStrategy {
    fn should_rotate(
        &self,
        state: &RotatingFileState,
        additional_bytes: usize,
    ) -> io::Result<bool> {
        match self {
            FileRotationStrategy::None => Ok(false),
            FileRotationStrategy::Size(cfg) => {
                Ok(state.bytes_written + additional_bytes as u64 >= cfg.max_size_bytes)
            }
            FileRotationStrategy::Interval(cfg) => {
                match SystemTime::now().duration_since(state.started_at) {
                    Ok(duration) => Ok(duration >= cfg.interval),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    fn rotate_file(&self, path: &Path) -> io::Result<Option<PathBuf>> {
        match self {
            FileRotationStrategy::None => Ok(None),
            FileRotationStrategy::Size(cfg) => rotate_file_with_timestamp(path, cfg.compress),
            FileRotationStrategy::Interval(cfg) => rotate_file_with_timestamp(path, cfg.compress),
        }
    }

    fn max_files(&self) -> Option<usize> {
        match self {
            FileRotationStrategy::None => None,
            FileRotationStrategy::Size(cfg) => cfg.max_files,
            FileRotationStrategy::Interval(cfg) => cfg.max_files,
        }
    }
}

struct RotatingFileWriter {
    path: PathBuf,
    append: bool,
    strategy: FileRotationStrategy,
    state: RotatingFileState,
}

struct RotatingFileState {
    file: Option<File>,
    bytes_written: u64,
    started_at: SystemTime,
}

impl RotatingFileWriter {
    fn new(path: PathBuf, append: bool, strategy: FileRotationStrategy) -> io::Result<Self> {
        let state = Self::open_initial_state(&path, append)?;
        Ok(Self {
            path,
            append,
            strategy,
            state,
        })
    }

    fn open_initial_state(path: &Path, append: bool) -> io::Result<RotatingFileState> {
        let (file, size) = open_log_file(path, append)?;
        Ok(RotatingFileState {
            file: Some(file),
            bytes_written: size,
            started_at: SystemTime::now(),
        })
    }

    fn maybe_rotate(&mut self, additional_bytes: usize) -> io::Result<()> {
        if self.strategy.should_rotate(&self.state, additional_bytes)? {
            self.rotate()?;
        }
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        if let Some(mut file) = self.state.file.take() {
            file.flush()?;
        }

        if let Some(_rotated_path) = self.strategy.rotate_file(&self.path)? {
            if let Some(max_files) = self.strategy.max_files() {
                prune_old_files(&self.path, max_files)?;
            }
        }

        self.state = Self::open_initial_state(&self.path, self.append)?;
        Ok(())
    }
}

impl Write for RotatingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.maybe_rotate(buf.len())?;
        if let Some(file) = self.state.file.as_mut() {
            let written = file.write(buf)?;
            self.state.bytes_written += written as u64;
            Ok(written)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "log file not initialized",
            ))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self.state.file.as_mut() {
            file.flush()
        } else {
            Ok(())
        }
    }
}

fn open_log_file(path: &Path, append: bool) -> io::Result<(File, u64)> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let file = options.open(path)?;
    let size = if append { file.metadata()?.len() } else { 0 };
    Ok((file, size))
}

fn rotate_file_with_timestamp(path: &Path, compress: bool) -> io::Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let mut candidate = build_rotated_path(path, &timestamp);
    let mut counter = 1;
    while candidate.exists() {
        let suffix = format!("{}-{}", timestamp, counter);
        candidate = build_rotated_path(path, &suffix);
        counter += 1;
    }

    fs::rename(path, &candidate)?;

    if compress {
        compress_log_file(&candidate).map(Some)
    } else {
        Ok(Some(candidate))
    }
}

fn build_rotated_path(base: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::new();
    name.push(base);
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

fn compress_log_file(path: &Path) -> io::Result<PathBuf> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut gz_path = OsString::new();
    gz_path.push(path);
    gz_path.push(".gz");
    let gz_path = PathBuf::from(gz_path);
    let gz_file = File::create(&gz_path)?;
    let mut encoder = GzEncoder::new(gz_file, Compression::default());
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    fs::remove_file(path)?;
    Ok(gz_path)
}

fn prune_old_files(base_path: &Path, max_files: usize) -> io::Result<()> {
    if max_files == 0 {
        return Ok(());
    }

    let dir = base_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let prefix = base_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| base_path.to_string_lossy().to_string());

    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str == prefix {
            continue;
        }
        if !name_str.starts_with(&format!("{}.", prefix)) {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        files.push((modified, entry.path()));
    }

    files.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, path) in files.into_iter().skip(max_files) {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

fn default_log_format() -> LogFormat {
    LogFormat::Full
}

fn default_log_rotation_max_size_bytes() -> u64 {
    10 * 1024 * 1024
}

fn default_log_rotation_max_files() -> usize {
    5
}

fn default_log_rotation_interval_hours() -> u64 {
    24
}

fn default_log_rotation_compress() -> bool {
    true
}
