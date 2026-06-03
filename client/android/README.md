# Capella Room - Android Client

Capella Room 的 Android 客户端项目。

## 项目结构

```
app/
├── src/main/
│   ├── java/com/capella/room/
│   │   ├── CapellaApplication.kt      # 应用入口
│   │   ├── data/                       # 数据层
│   │   │   ├── api/                    # API 接口
│   │   │   ├── local/                  # 本地数据库
│   │   │   ├── model/                  # 数据模型
│   │   │   └── repository/             # 仓库层
│   │   ├── ui/                         # UI 层
│   │   │   ├── auth/                   # 认证相关
│   │   │   ├── channel/                # 频道列表
│   │   │   ├── chat/                   # 聊天界面
│   │   │   ├── profile/                # 个人资料
│   │   │   └── MainActivity.kt         # 主活动
│   │   └── util/                       # 工具类
│   └── res/                            # 资源文件
│       ├── layout/                     # 布局文件
│       ├── values/                     # 值资源
│       └── drawable/                   # 图片资源
```

## 技术栈

- **语言**: Kotlin
- **UI**: XML Layout + Material Design 3
- **架构**: MVVM
- **网络**: Retrofit2 + Gson
- **异步**: Kotlin Coroutines
- **图片**: Coil
- **存储**: DataStore

## 开发环境要求

- Android Studio Hedgehog (2023.1.1) 或更高版本
- JDK 17
- Android SDK 34
- Gradle 8.2

## 原型参考

原型文件位于 `../../prototype/android_app/`：
- `login.html` - 登录页设计
- `index.html` - 主页/频道列表
- `channels.html` - 频道页
- `chat.html` - 聊天界面
- `thread.html` - 话题页
- `profile.html` - 个人资料

## 设计规范

- **主背景**: `#0B0B14`
- **表面色**: `#141420`
- **强调色**: `#7C5CFC`
- **文字主色**: `#ECECF1`
- **文字次要**: `#8B8B9E`
- **边框色**: `#262636`

## 构建

```bash
./gradlew assembleDebug
```

## 待办事项

- [ ] 实现登录/注册功能
- [ ] 实现频道列表
- [ ] 实现聊天界面
- [ ] 实现 WebSocket 连接
- [ ] 实现个人资料页
