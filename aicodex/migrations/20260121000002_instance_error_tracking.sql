-- 添加实例错误跟踪字段
-- 用于在前端显示启动/停止失败的错误信息

ALTER TABLE vibe_instances ADD COLUMN last_error TEXT;
ALTER TABLE vibe_instances ADD COLUMN last_error_at DATETIME;
