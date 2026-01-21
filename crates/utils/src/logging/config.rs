//! 日志系统配置模块

use std::path::PathBuf;

/// 日志输出格式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// 人类可读格式（带颜色）
    #[default]
    Pretty,
    /// JSON 结构化格式
    Json,
}

impl LogFormat {
    /// 从字符串解析日志格式
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pretty" => Some(Self::Pretty),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// 日志输出目标
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogOutput {
    /// 仅控制台输出
    #[default]
    Console,
    /// 仅文件输出
    File,
    /// 同时输出到控制台和文件
    Both,
}

impl LogOutput {
    /// 从字符串解析输出目标
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "console" => Some(Self::Console),
            "file" => Some(Self::File),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// 日志轮转策略
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogRotation {
    /// 每日轮转
    #[default]
    Daily,
    /// 每小时轮转
    Hourly,
}

impl LogRotation {
    /// 从字符串解析轮转策略
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "daily" => Some(Self::Daily),
            "hourly" => Some(Self::Hourly),
            _ => None,
        }
    }
}

/// 日志系统配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 默认日志级别
    pub level: String,
    /// 输出格式
    pub format: LogFormat,
    /// 输出目标
    pub output: LogOutput,
    /// 日志文件目录
    pub log_dir: PathBuf,
    /// 最大保留文件数
    pub max_files: usize,
    /// 轮转策略
    pub rotation: LogRotation,
    /// 自定义脱敏正则表达式
    pub redact_patterns: Vec<String>,
    /// 服务名称（用于日志文件名）
    pub service_name: String,
    /// 是否启用 Sentry 集成
    pub enable_sentry: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Console,
            log_dir: PathBuf::from("./logs"),
            max_files: 7,
            rotation: LogRotation::Daily,
            redact_patterns: Vec::new(),
            service_name: "app".to_string(),
            enable_sentry: false,
        }
    }
}

impl LoggingConfig {
    /// 创建新的配置构建器
    pub fn builder() -> LoggingConfigBuilder {
        LoggingConfigBuilder::default()
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // LOG_LEVEL
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.level = level;
        }

        // LOG_FORMAT
        if let Ok(format) = std::env::var("LOG_FORMAT") {
            if let Some(f) = LogFormat::from_str(&format) {
                config.format = f;
            } else {
                tracing::warn!("无效的 LOG_FORMAT 值: {}，使用默认值 pretty", format);
            }
        }

        // LOG_OUTPUT
        if let Ok(output) = std::env::var("LOG_OUTPUT") {
            if let Some(o) = LogOutput::from_str(&output) {
                config.output = o;
            } else {
                tracing::warn!("无效的 LOG_OUTPUT 值: {}，使用默认值 console", output);
            }
        }

        // LOG_DIR
        if let Ok(dir) = std::env::var("LOG_DIR") {
            config.log_dir = PathBuf::from(dir);
        }

        // LOG_MAX_FILES
        if let Ok(max) = std::env::var("LOG_MAX_FILES") {
            if let Ok(n) = max.parse::<usize>() {
                config.max_files = n;
            } else {
                tracing::warn!("无效的 LOG_MAX_FILES 值: {}，使用默认值 7", max);
            }
        }

        // LOG_ROTATION
        if let Ok(rotation) = std::env::var("LOG_ROTATION") {
            if let Some(r) = LogRotation::from_str(&rotation) {
                config.rotation = r;
            } else {
                tracing::warn!("无效的 LOG_ROTATION 值: {}，使用默认值 daily", rotation);
            }
        }

        // LOG_REDACT_PATTERNS
        if let Ok(patterns) = std::env::var("LOG_REDACT_PATTERNS") {
            config.redact_patterns = patterns
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        config
    }

    /// 检查是否需要输出到控制台
    pub fn should_output_console(&self) -> bool {
        matches!(self.output, LogOutput::Console | LogOutput::Both)
    }

    /// 检查是否需要输出到文件
    pub fn should_output_file(&self) -> bool {
        matches!(self.output, LogOutput::File | LogOutput::Both)
    }

    /// 检查是否使用 JSON 格式
    pub fn is_json_format(&self) -> bool {
        matches!(self.format, LogFormat::Json)
    }
}

/// 配置构建器
#[derive(Debug, Default)]
pub struct LoggingConfigBuilder {
    config: LoggingConfig,
}

impl LoggingConfigBuilder {
    /// 设置日志级别
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.config.level = level.into();
        self
    }

    /// 设置输出格式
    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    /// 设置输出目标
    pub fn output(mut self, output: LogOutput) -> Self {
        self.config.output = output;
        self
    }

    /// 设置日志目录
    pub fn log_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.log_dir = dir.into();
        self
    }

    /// 设置最大文件数
    pub fn max_files(mut self, max: usize) -> Self {
        self.config.max_files = max;
        self
    }

    /// 设置轮转策略
    pub fn rotation(mut self, rotation: LogRotation) -> Self {
        self.config.rotation = rotation;
        self
    }

    /// 设置服务名称
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    /// 启用 Sentry
    pub fn enable_sentry(mut self, enable: bool) -> Self {
        self.config.enable_sentry = enable;
        self
    }

    /// 添加脱敏模式
    pub fn add_redact_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.config.redact_patterns.push(pattern.into());
        self
    }

    /// 从环境变量合并配置（环境变量优先）
    pub fn with_env(mut self) -> Self {
        let env_config = LoggingConfig::from_env();

        // 环境变量覆盖代码配置
        if std::env::var("LOG_LEVEL").is_ok() {
            self.config.level = env_config.level;
        }
        if std::env::var("LOG_FORMAT").is_ok() {
            self.config.format = env_config.format;
        }
        if std::env::var("LOG_OUTPUT").is_ok() {
            self.config.output = env_config.output;
        }
        if std::env::var("LOG_DIR").is_ok() {
            self.config.log_dir = env_config.log_dir;
        }
        if std::env::var("LOG_MAX_FILES").is_ok() {
            self.config.max_files = env_config.max_files;
        }
        if std::env::var("LOG_ROTATION").is_ok() {
            self.config.rotation = env_config.rotation;
        }
        if std::env::var("LOG_REDACT_PATTERNS").is_ok() {
            self.config.redact_patterns.extend(env_config.redact_patterns);
        }

        self
    }

    /// 构建配置
    pub fn build(self) -> LoggingConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Pretty);
        assert_eq!(config.output, LogOutput::Console);
        assert_eq!(config.max_files, 7);
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!(LogFormat::from_str("pretty"), Some(LogFormat::Pretty));
        assert_eq!(LogFormat::from_str("PRETTY"), Some(LogFormat::Pretty));
        assert_eq!(LogFormat::from_str("json"), Some(LogFormat::Json));
        assert_eq!(LogFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_output_parsing() {
        assert_eq!(LogOutput::from_str("console"), Some(LogOutput::Console));
        assert_eq!(LogOutput::from_str("file"), Some(LogOutput::File));
        assert_eq!(LogOutput::from_str("both"), Some(LogOutput::Both));
        assert_eq!(LogOutput::from_str("invalid"), None);
    }

    #[test]
    fn test_builder() {
        let config = LoggingConfig::builder()
            .level("debug")
            .format(LogFormat::Json)
            .output(LogOutput::Both)
            .max_files(14)
            .service_name("test-service")
            .build();

        assert_eq!(config.level, "debug");
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.output, LogOutput::Both);
        assert_eq!(config.max_files, 14);
        assert_eq!(config.service_name, "test-service");
    }
}
