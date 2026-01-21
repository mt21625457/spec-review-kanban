# aicodex 后端服务 Dockerfile
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# 构建发布版本
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/aicodex /app/aicodex
COPY --from=builder /app/migrations /app/migrations

# 创建数据目录
RUN mkdir -p /data

ENV DATABASE_URL=sqlite:///data/aicodex.db?mode=rwc
ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["/app/aicodex"]
