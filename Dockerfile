# =============================================================================
# Capella Room - Production Dockerfile
# 使用 cargo-chef + sparse 索引加速构建
# 镜像源: 阿里云
# =============================================================================

# =============================================================================
# 阶段 1: 准备依赖缓存 (cargo-chef)
# =============================================================================
FROM rust:1.94-slim-bookworm AS chef

WORKDIR /app

# 配置 APT 镜像源（使用阿里云镜像加速）
RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
    sed -i 's/security.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources

# 配置 Cargo 稀疏索引 + 阿里云镜像源 + mold 链接器（必须在安装 cargo-chef 之前）
RUN mkdir -p /usr/local/cargo && \
    cat <<'EOF' > /usr/local/cargo/config.toml
[registries.crates-io]
protocol = "sparse"

[source.crates-io]
replace-with = "aliyun-sparse"

[registries.aliyun-sparse]
index = "sparse+https://mirrors.aliyun.com/crates.io-index/"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF

# 设置环境变量确保 Cargo 能读取配置
ENV CARGO_HOME=/usr/local/cargo

# 安装 cargo-chef + mold 链接器（使用阿里云源加速）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    mold \
    clang \
    && rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef

# =============================================================================
# 阶段 2: 生成依赖配方
# =============================================================================
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# 阶段 3: 构建应用
# =============================================================================
FROM chef AS builder

# 确保 Cargo 配置在 builder 阶段也可用
ENV CARGO_HOME=/usr/local/cargo

# 从 planner 复制配方
COPY --from=planner /app/recipe.json recipe.json

# 编译依赖（使用 BuildKit 缓存，共享 target 实现增量编译）
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    cargo chef cook --release --recipe-path recipe.json

# 复制源代码并编译
COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    cargo build --release && \
    cp target/release/capella-room /app/server

# =============================================================================
# 阶段 4: 运行器
# =============================================================================
FROM debian:bookworm-slim AS runner

WORKDIR /app

# 配置 APT 镜像源（使用阿里云镜像加速）
RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
    sed -i 's/security.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN groupadd -r appuser && useradd -r -g appuser -d /app -s /sbin/nologin appuser

# 创建上传目录
RUN mkdir -p /app/uploads && chown -R appuser:appuser /app/uploads

# 运行时以 user: "1000:1000" 运行（宿主机用户 UID），日志目录需对所有用户可写
RUN mkdir -p /app/logs && chmod 777 /app/logs

# 从构建阶段复制二进制文件
COPY --from=builder /app/server /app/server

# 复制配置文件
COPY --from=builder /app/config.toml /app/config.toml

# 设置权限
RUN chown -R appuser:appuser /app

# 复制入口脚本
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 运行时以 root 启动，entrypoint 修复上传目录权限后自动降权到 appuser
USER root
ENTRYPOINT ["/app/entrypoint.sh"]
