//! 敏感信息脱敏模块

use regex::Regex;
use std::borrow::Cow;
use std::sync::OnceLock;

/// 脱敏后的占位符
pub const REDACTED: &str = "[REDACTED]";
pub const REDACTED_JWT: &str = "[REDACTED:JWT]";
pub const REDACTED_BEARER: &str = "[REDACTED:BEARER]";
pub const REDACTED_API_KEY: &str = "[REDACTED:API_KEY]";

/// 敏感字段名关键词（小写）
const SENSITIVE_FIELD_KEYWORDS: &[&str] = &[
    "password",
    "passwd",
    "token",
    "secret",
    "key",
    "credential",
    "authorization",
    "api_key",
    "apikey",
    "auth",
    "private",
];

/// 预编译的敏感模式正则表达式
static JWT_PATTERN: OnceLock<Regex> = OnceLock::new();
static BEARER_PATTERN: OnceLock<Regex> = OnceLock::new();
static API_KEY_PATTERN: OnceLock<Regex> = OnceLock::new();

fn jwt_pattern() -> &'static Regex {
    JWT_PATTERN.get_or_init(|| {
        // JWT 格式: eyJ 开头，包含两个点分隔的三部分
        Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap()
    })
}

fn bearer_pattern() -> &'static Regex {
    BEARER_PATTERN.get_or_init(|| {
        // Bearer token 格式
        Regex::new(r"Bearer\s+[A-Za-z0-9_-]+").unwrap()
    })
}

fn api_key_pattern() -> &'static Regex {
    API_KEY_PATTERN.get_or_init(|| {
        // API Key: 32+ 字符的十六进制或 Base64
        Regex::new(r"[A-Fa-f0-9]{32,}|[A-Za-z0-9+/]{32,}={0,2}").unwrap()
    })
}

/// 敏感信息脱敏器
#[derive(Debug, Clone)]
pub struct Redactor {
    /// 自定义脱敏正则表达式
    custom_patterns: Vec<Regex>,
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Redactor {
    /// 创建新的脱敏器
    pub fn new() -> Self {
        Self {
            custom_patterns: Vec::new(),
        }
    }

    /// 添加自定义脱敏模式
    pub fn with_patterns(mut self, patterns: &[String]) -> Self {
        for pattern in patterns {
            match Regex::new(pattern) {
                Ok(regex) => self.custom_patterns.push(regex),
                Err(e) => {
                    tracing::warn!("无效的脱敏正则表达式 '{}': {}", pattern, e);
                }
            }
        }
        self
    }

    /// 检查字段名是否敏感
    pub fn is_sensitive_field(field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        SENSITIVE_FIELD_KEYWORDS
            .iter()
            .any(|keyword| lower.contains(keyword))
    }

    /// 对字符串进行脱敏处理
    pub fn redact<'a>(&self, input: &'a str) -> Cow<'a, str> {
        let mut result = Cow::Borrowed(input);

        // 1. JWT token 脱敏
        if jwt_pattern().is_match(&result) {
            result = Cow::Owned(jwt_pattern().replace_all(&result, REDACTED_JWT).into_owned());
        }

        // 2. Bearer token 脱敏
        if bearer_pattern().is_match(&result) {
            result = Cow::Owned(
                bearer_pattern()
                    .replace_all(&result, REDACTED_BEARER)
                    .into_owned(),
            );
        }

        // 3. 自定义模式脱敏
        for pattern in &self.custom_patterns {
            if pattern.is_match(&result) {
                result = Cow::Owned(pattern.replace_all(&result, REDACTED).into_owned());
            }
        }

        result
    }

    /// 对包含敏感字段的日志行进行脱敏
    pub fn redact_log_line<'a>(&self, line: &'a str) -> Cow<'a, str> {
        // 首先应用基本脱敏
        let result = self.redact(line);

        // 检查是否包含敏感字段名（简单的键值对检测）
        // 格式如: password=xxx, "token": "xxx", secret: xxx
        let mut output = result.into_owned();

        for keyword in SENSITIVE_FIELD_KEYWORDS {
            // 处理 key=value 格式
            let pattern_eq = format!(r#"(?i){}(\s*=\s*)[^\s,\}}\]]+"#, regex::escape(keyword));
            if let Ok(re) = Regex::new(&pattern_eq) {
                output = re
                    .replace_all(&output, format!("{}$1{}", keyword, REDACTED))
                    .into_owned();
            }

            // 处理 "key": "value" JSON 格式
            let pattern_json = format!(
                r#"(?i)"{}"\s*:\s*"[^"]*""#,
                regex::escape(keyword)
            );
            if let Ok(re) = Regex::new(&pattern_json) {
                output = re
                    .replace_all(&output, format!(r#""{}": "{}""#, keyword, REDACTED))
                    .into_owned();
            }
        }

        if output == line {
            Cow::Borrowed(line)
        } else {
            Cow::Owned(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_field_detection() {
        assert!(Redactor::is_sensitive_field("password"));
        assert!(Redactor::is_sensitive_field("Password"));
        assert!(Redactor::is_sensitive_field("user_password"));
        assert!(Redactor::is_sensitive_field("api_key"));
        assert!(Redactor::is_sensitive_field("API_KEY"));
        assert!(Redactor::is_sensitive_field("secret_token"));
        assert!(Redactor::is_sensitive_field("authorization"));

        assert!(!Redactor::is_sensitive_field("username"));
        assert!(!Redactor::is_sensitive_field("email"));
        assert!(!Redactor::is_sensitive_field("id"));
    }

    #[test]
    fn test_jwt_redaction() {
        let redactor = Redactor::new();
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = redactor.redact(jwt);
        assert_eq!(result, REDACTED_JWT);
    }

    #[test]
    fn test_bearer_redaction() {
        let redactor = Redactor::new();
        let bearer = "Bearer abc123xyz";
        let result = redactor.redact(bearer);
        assert_eq!(result, REDACTED_BEARER);
    }

    #[test]
    fn test_log_line_redaction() {
        let redactor = Redactor::new();

        // 测试 key=value 格式
        let line = "User login with password=secret123 successful";
        let result = redactor.redact_log_line(line);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("secret123"));

        // 测试 JSON 格式
        let json_line = r#"{"user": "admin", "token": "abc123"}"#;
        let result = redactor.redact_log_line(json_line);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("abc123"));
    }

    #[test]
    fn test_custom_pattern() {
        let redactor = Redactor::new().with_patterns(&["\\d{4}-\\d{4}-\\d{4}".to_string()]);

        let line = "Credit card: 1234-5678-9012";
        let result = redactor.redact(line);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("1234-5678-9012"));
    }
}
