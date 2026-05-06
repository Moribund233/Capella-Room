# 部署文档

本文档说明 Seredeli Room 用户端的生产环境部署方式。

## 部署方式

### 方式一：静态文件托管（推荐）

构建后生成静态文件，可部署到任何静态文件托管服务。

#### 1. 构建生产版本

```bash
# 安装依赖
pnpm install

# 构建生产版本
pnpm build
```

构建完成后，`dist/` 目录包含所有静态文件。

#### 2. 部署到 Nginx

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /path/to/dist;
    index index.html;

    # Gzip 压缩
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json application/javascript text/xml;

    # 缓存静态资源
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # 前端路由支持
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API 代理
    location /api/ {
        proxy_pass http://backend-server:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket 代理
    location /ws {
        proxy_pass http://backend-server:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

#### 3. 部署到 CDN

```bash
# 阿里云 OSS
ossutil cp -r dist/ oss://your-bucket/

# AWS S3
aws s3 sync dist/ s3://your-bucket/
```

### 方式二：Docker 部署

#### 1. 构建 Docker 镜像

```dockerfile
# Dockerfile
FROM node:22-alpine AS builder

WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile

COPY . .
RUN pnpm build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
```

#### 2. 构建并运行

```bash
# 构建镜像
docker build -t seredeli-room-user .

# 运行容器
docker run -d -p 80:80 --name seredeli-user seredeli-room-user
```

#### 3. Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  user-client:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "80:80"
    restart: unless-stopped
```

### 方式三：Vercel / Netlify 部署

#### Vercel

```bash
# 安装 Vercel CLI
npm i -g vercel

# 部署
vercel --prod
```

配置 `vercel.json`：

```json
{
  "rewrites": [
    { "source": "/(.*)", "destination": "/index.html" }
  ]
}
```

#### Netlify

```bash
# 安装 Netlify CLI
npm i -g netlify-cli

# 部署
netlify deploy --prod --dir=dist
```

配置 `netlify.toml`：

```toml
[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

## 环境变量配置

### 开发环境

```env
# .env.development
VITE_API_BASE_URL=http://localhost:8080/api/v1
VITE_WS_URL=ws://localhost:8080/ws
```

### 生产环境

```env
# .env.production
VITE_API_BASE_URL=https://api.your-domain.com/api/v1
VITE_WS_URL=wss://api.your-domain.com/ws
```

## 性能优化

### 构建优化

```bash
# 分析构建体积
pnpm build -- --mode analyze

# 查看报告
npx serve dist/report
```

### 启用 Brotli 压缩

```nginx
# nginx.conf
brotli on;
brotli_comp_level 6;
brotli_types text/plain text/css application/json application/javascript;
```

### 图片优化

- 使用 WebP 格式
- 启用懒加载
- 使用 CDN 图片处理服务

## 监控与日志

### 前端监控

```typescript
// 错误上报
window.addEventListener('error', (e) => {
  // 上报到监控系统
  reportError({
    message: e.message,
    filename: e.filename,
    lineno: e.lineno,
    colno: e.colno,
  })
})

// 性能监控
const observer = new PerformanceObserver((list) => {
  for (const entry of list.getEntries()) {
    console.log('Performance entry:', entry)
  }
})
observer.observe({ entryTypes: ['measure', 'navigation'] })
```

### Nginx 日志

```nginx
log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                '$status $body_bytes_sent "$http_referer" '
                '"$http_user_agent" "$http_x_forwarded_for" '
                '$request_time $upstream_response_time';

access_log /var/log/nginx/access.log main;
```

## 安全建议

### HTTPS 配置

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # HSTS
    add_header Strict-Transport-Security "max-age=31536000" always;
}
```

### 安全响应头

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

## 回滚策略

### 版本标记

```bash
# 构建时添加版本号
VITE_APP_VERSION=$(git describe --tags --always) pnpm build
```

### 蓝绿部署

```bash
# 部署新版本到绿色环境
docker-compose -f docker-compose.green.yml up -d

# 健康检查
curl -f http://green-env/health || exit 1

# 切换流量
ln -sfn green-env current-env
```

## 故障排查

### 常见问题

1. **404 刷新页面**
   - 检查 Nginx 配置 `try_files`
   - 确认路由模式为 `history` 模式

2. **WebSocket 连接失败**
   - 检查代理配置
   - 确认防火墙允许 WebSocket

3. **API 请求跨域**
   - 配置后端 CORS
   - 或使用 Nginx 代理

4. **资源加载 404**
   - 检查 `base` 配置
   - 确认资源路径正确

### 调试命令

```bash
# 检查 Nginx 配置
nginx -t

# 查看错误日志
tail -f /var/log/nginx/error.log

# 检查端口占用
netstat -tlnp | grep 80

# 测试 API 连通性
curl -v https://api.your-domain.com/health
```

## 更新维护

### 自动更新检查

```typescript
// 检查新版本
const checkUpdate = async () => {
  const res = await fetch('/version.json')
  const { version } = await res.json()
  if (version !== CURRENT_VERSION) {
    // 提示用户刷新
    showUpdateNotification()
  }
}
```

### 平滑升级

1. 部署新版本到独立目录
2. 验证新版本功能正常
3. 切换 Nginx 指向新目录
4. 保留旧版本 24 小时以备回滚
