# Capella Room - Kubernetes GitOps 部署

由 Rancher Fleet 从本仓库自动同步（`deploy/prod/` → local 集群 `capella` 命名空间）。

## 结构

- `namespace.yaml` - `capella` 命名空间
- `postgres.yaml` - PostgreSQL 16 StatefulSet + 服务
- `redis.yaml` - Redis 7 StatefulSet + 服务
- `app.yaml` - 应用 Deployment + uploads PVC + NodePort(30081)

## 敏感配置（不在 git 中）

以下配置通过集群内 Secret `capella/capella-env` 注入，由运维创建，不得提交到仓库：

| Key | 用途 |
|---|---|
| `POSTGRES_PASSWORD` | PostgreSQL 密码 |
| `DATABASE_URL` | `postgres://capella:<pw>@capella-postgres:5432/capella_room` |
| `JWT_SECRET` | JWT 签名密钥 |
| `OAUTH_JWT_SECRET` | OAuth JWT 签名密钥 |
| `ADMIN_INITIAL_PASSWORD` | 初始管理员密码 |
| `REDIS_ENABLED` / `REDIS_URL` | Redis 开关与地址 |
| `UPLOAD_DIR` | 上传目录 |
| `CLUSTER_ID` / `CLUSTER_NAME` | 集群标识（多节点共享） |

另有 `capella/acr-secret`（dockerconfigjson）供集群从阿里云 ACR 拉取镜像。

## 镜像

`crpi-a7uhzxza3co1oh1t.cn-shenzhen.personal.cr.aliyuncs.com/moribund-projects/capella-room:<tag>`

升级：改 `app.yaml` 的镜像 tag → 提交 → Fleet 自动滚动更新（tag 使用不可变版本号，如 `v0.1.0`）。