# =============================================================================
# Seredeli Room - 优化构建版本 Dockerfile
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

# 安装 cargo-chef（不使用 --locked 以避免版本兼容问题）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef

# 配置 Cargo 稀疏索引 + 阿里云镜像源
RUN mkdir -p $HOME/.cargo && \
    cat <<'EOF' > $HOME/.cargo/config.toml
[registries.crates-io]
protocol = "sparse"

[source.crates-io]
replace-with = "aliyun-sparse"

[registries.aliyun-sparse]
index = "sparse+https://mirrors.aliyun.com/crates.io-index/"
EOF

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

# 从 planner 复制配方
COPY --from=planner /app/recipe.json recipe.json

# 编译依赖（这一步会被缓存，除非 recipe.json 变化）
RUN cargo chef cook --release --recipe-path recipe.json

# 复制源代码并编译
COPY . .
RUN cargo build --release --locked

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

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/seredeli-room /app/server

# 设置权限
RUN chown -R appuser:appuser /app

# 切换到非 root 用户
USER appuser

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动命令
ENTRYPOINT ["/app/server"]
