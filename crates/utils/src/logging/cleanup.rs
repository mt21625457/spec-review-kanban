//! 日志文件清理模块

use std::fs;
use std::io;
use std::path::Path;

/// 清理旧日志文件
///
/// 删除超过 `max_files` 数量的旧日志文件。
/// 按文件修改时间排序，删除最旧的文件。
pub fn cleanup_old_logs(log_dir: &Path, prefix: &str, max_files: usize) -> io::Result<usize> {
    if !log_dir.exists() {
        return Ok(0);
    }

    // 收集所有匹配的日志文件
    let mut log_files: Vec<_> = fs::read_dir(log_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return false;
            }
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            file_name.starts_with(prefix) && file_name.ends_with(".log")
        })
        .collect();

    // 如果文件数量未超过限制，无需清理
    if log_files.len() <= max_files {
        return Ok(0);
    }

    // 按修改时间排序（最旧的在前）
    log_files.sort_by(|a, b| {
        let time_a = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_a.cmp(&time_b)
    });

    // 计算需要删除的文件数量
    let files_to_remove = log_files.len() - max_files;
    let mut removed_count = 0;

    // 删除最旧的文件
    for entry in log_files.iter().take(files_to_remove) {
        let path = entry.path();
        match fs::remove_file(&path) {
            Ok(()) => {
                tracing::info!("已删除旧日志文件: {:?}", path);
                removed_count += 1;
            }
            Err(e) => {
                tracing::warn!("删除日志文件失败 {:?}: {}", path, e);
            }
        }
    }

    Ok(removed_count)
}

/// 确保日志目录存在
///
/// 如果目录不存在则创建，返回是否创建成功。
pub fn ensure_log_dir(log_dir: &Path) -> io::Result<bool> {
    if log_dir.exists() {
        if log_dir.is_dir() {
            return Ok(false);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{:?} 存在但不是目录", log_dir),
            ));
        }
    }

    fs::create_dir_all(log_dir)?;
    tracing::info!("创建日志目录: {:?}", log_dir);
    Ok(true)
}

/// 检查目录是否有写入权限
pub fn check_write_permission(log_dir: &Path) -> io::Result<bool> {
    if !log_dir.exists() {
        return Ok(true); // 目录不存在时假设可以创建
    }

    // 尝试创建临时文件来测试写入权限
    let test_file = log_dir.join(".write_test");
    match fs::write(&test_file, "test") {
        Ok(()) => {
            let _ = fs::remove_file(&test_file);
            Ok(true)
        }
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_cleanup_old_logs() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // 创建测试日志文件
        for i in 0..5 {
            let file_path = log_dir.join(format!("app.2024-01-{:02}.log", i + 1));
            File::create(&file_path).unwrap();
            // 添加小延迟以确保不同的修改时间
            sleep(Duration::from_millis(10));
        }

        // 清理，保留 3 个文件
        let removed = cleanup_old_logs(log_dir, "app", 3).unwrap();
        assert_eq!(removed, 2);

        // 验证剩余文件数量
        let remaining: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
            .collect();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn test_ensure_log_dir() {
        let temp_dir = tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_logs");

        assert!(!new_dir.exists());
        let created = ensure_log_dir(&new_dir).unwrap();
        assert!(created);
        assert!(new_dir.exists());

        // 再次调用应返回 false
        let created = ensure_log_dir(&new_dir).unwrap();
        assert!(!created);
    }

    #[test]
    fn test_check_write_permission() {
        let temp_dir = tempdir().unwrap();
        let has_permission = check_write_permission(temp_dir.path()).unwrap();
        assert!(has_permission);
    }
}
