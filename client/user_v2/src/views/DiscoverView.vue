<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NavBar } from '@/components/nav'
import { QuickBar } from '@/components/quick'
import type { QuickItem } from '@/components/quick'
import { useTheme } from '@/composables/useTheme'
import {
  Search,
  Bell,
  Moon,
  Sunny,
  User,
  TrendCharts,
  CollectionTag,
  Lock,
  OfficeBuilding,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const { isDark, toggleTheme } = useTheme()

// QuickBar 配置 - 主题图标根据当前主题动态变化
const quickItems = computed<QuickItem[]>(() => [
  {
    key: 'notifications',
    display: 'visible',
    icon: Bell,
    label: t('profile.preferences.notifications.title'),
    badge: 3,
    onClick: () => {},
  },
  {
    key: 'theme',
    display: 'visible',
    icon: isDark.value ? Moon : Sunny,
    label: isDark.value ? t('profile.appearance.theme.dark') : t('profile.appearance.theme.light'),
    onClick: toggleTheme,
  },
])

// 搜索关键词
const searchQuery = ref('')

// 分类标签
const categories = [
  { key: 'all', label: t('discover.all') },
  { key: 'popular', label: t('discover.popular') },
  { key: 'tech', label: t('discover.tech') },
  { key: 'gaming', label: t('discover.gaming') },
  { key: 'music', label: t('discover.music') },
  { key: 'art', label: t('discover.art') },
]

const activeCategory = ref('all')

// 推荐房间
const featuredRooms = [
  {
    id: 'vue-community',
    name: 'Vue.js Community',
    description: 'Official Vue.js community for developers and enthusiasts',
    members: 12580,
    tags: ['tech', 'javascript', 'vue'],
    type: 'public',
    icon: 'V',
    iconColor: 'var(--wave-accent-green)',
  },
  {
    id: 'rust-lang',
    name: 'Rust Programming',
    description: 'A place for Rustaceans to learn, share, and discuss',
    members: 8932,
    tags: ['tech', 'rust', 'systems'],
    type: 'public',
    icon: 'R',
    iconColor: 'var(--wave-accent-orange)',
  },
  {
    id: 'design-systems',
    name: 'Design Systems',
    description: 'Discussing design systems, component libraries, and UI patterns',
    members: 5640,
    tags: ['design', 'ui', 'ux'],
    type: 'public',
    icon: 'D',
    iconColor: 'var(--wave-accent-pink)',
  },
]

// 热门房间
const trendingRooms = [
  {
    id: 'gamedev',
    name: 'Game Development',
    description: 'Indie and professional game developers',
    members: 4210,
    growth: '+12%',
    type: 'public',
  },
  {
    id: 'ai-ml',
    name: 'AI & Machine Learning',
    description: 'Discussing the latest in AI and ML',
    members: 3890,
    growth: '+28%',
    type: 'public',
  },
  {
    id: 'web3',
    name: 'Web3 & Blockchain',
    description: 'Decentralized web and blockchain tech',
    members: 2150,
    growth: '+8%',
    type: 'public',
  },
  {
    id: 'creative-coding',
    name: 'Creative Coding',
    description: 'Generative art and creative programming',
    members: 1840,
    growth: '+15%',
    type: 'public',
  },
]

/**
 * 加入房间
 */
function joinRoom() {
  // TODO: 实现加入房间逻辑
  router.push(`/app`)
}

/**
 * 查看房间详情
 */
function viewRoom() {
  // TODO: 实现查看房间详情逻辑
}
</script>

<template>
  <div class="discover-layout">
    <!-- 窄边导航栏 -->
    <NavBar>
      <template #quick-bar>
        <QuickBar :items="quickItems" />
      </template>
    </NavBar>

    <!-- 主内容区 -->
    <main class="discover-main">
      <!-- 头部 -->
      <header class="discover-header">
        <div class="header-content">
          <h1 class="discover-title">{{ t('discover.title') }}</h1>
          <p class="discover-subtitle">{{ t('discover.subtitle') }}</p>

          <!-- 搜索框 -->
          <div class="search-box">
            <el-input
              v-model="searchQuery"
              :placeholder="t('discover.searchPlaceholder')"
              :prefix-icon="Search"
              size="large"
              class="search-input"
            />
          </div>

          <!-- 分类标签 -->
          <div class="category-tabs">
            <button
              v-for="category in categories"
              :key="category.key"
              class="category-tab"
              :class="{ active: activeCategory === category.key }"
              @click="activeCategory = category.key"
            >
              {{ category.label }}
            </button>
          </div>
        </div>
      </header>

      <!-- 内容区 -->
      <div class="discover-content">
        <!-- 推荐房间 -->
        <section class="content-section">
          <h2 class="section-title">
            <el-icon><TrendCharts /></el-icon>
            {{ t('discover.featured') }}
          </h2>
          <div class="featured-grid">
            <el-card
              v-for="room in featuredRooms"
              :key="room.id"
              class="room-card featured"
              shadow="never"
            >
              <div class="room-card-header">
                <div class="room-icon" :style="{ background: room.iconColor }">
                  {{ room.icon }}
                </div>
                <div class="room-info">
                  <h3 class="room-name">{{ room.name }}</h3>
                  <div class="room-meta">
                    <el-icon><User /></el-icon>
                    <span>{{ room.members.toLocaleString() }} {{ t('chat.members') }}</span>
                    <el-icon v-if="room.type === 'public'"><OfficeBuilding /></el-icon>
                    <el-icon v-else><Lock /></el-icon>
                  </div>
                </div>
              </div>
              <p class="room-description">{{ room.description }}</p>
              <div class="room-tags">
                <el-tag v-for="tag in room.tags" :key="tag" size="small" effect="plain">
                  #{{ tag }}
                </el-tag>
              </div>
              <div class="room-actions">
                <el-button type="primary" @click="joinRoom()">
                  {{ t('discover.join') }}
                </el-button>
                <el-button text @click="viewRoom()">
                  {{ t('discover.view') }}
                </el-button>
              </div>
            </el-card>
          </div>
        </section>

        <!-- 热门房间 -->
        <section class="content-section">
          <h2 class="section-title">
            <el-icon><CollectionTag /></el-icon>
            {{ t('discover.trending') }}
          </h2>
          <div class="trending-list">
            <el-card
              v-for="room in trendingRooms"
              :key="room.id"
              class="room-card trending"
              shadow="never"
            >
              <div class="trending-content">
                <div class="trending-info">
                  <h3 class="room-name">{{ room.name }}</h3>
                  <p class="room-description">{{ room.description }}</p>
                  <div class="room-meta">
                    <el-icon><User /></el-icon>
                    <span>{{ room.members.toLocaleString() }} {{ t('chat.members') }}</span>
                    <span class="growth">{{ room.growth }}</span>
                  </div>
                </div>
                <el-button type="primary" @click="joinRoom()">
                  {{ t('discover.join') }}
                </el-button>
              </div>
            </el-card>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<style scoped lang="scss">
.discover-layout {
  display: flex;
  height: 100vh;
  background: var(--wave-bg);
  color: var(--wave-fg);
  overflow: hidden;
}

.discover-main {
  flex: 1;
  overflow-y: auto;
}

.discover-header {
  padding: 48px 48px 32px;
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--wave-accent) 8%, var(--wave-bg)) 0%,
    var(--wave-bg) 100%
  );
  border-bottom: 1px solid var(--wave-border);
}

.header-content {
  max-width: 960px;
  margin: 0 auto;
}

.discover-title {
  font-family: var(--wave-font-display);
  font-size: 36px;
  font-weight: 700;
  margin: 0 0 8px;
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.discover-subtitle {
  font-size: 16px;
  color: var(--wave-muted);
  margin: 0 0 24px;
}

.search-box {
  margin-bottom: 24px;
}

.search-input {
  :deep(.el-input__wrapper) {
    background-color: var(--wave-surface);
    border: 1px solid var(--wave-border);
    border-radius: var(--wave-radius-lg);
    padding: 8px 16px;
    box-shadow: none;

    &.is-focus {
      border-color: var(--wave-accent);
    }
  }

  :deep(.el-input__inner) {
    font-size: 16px;
    height: 40px;
  }
}

.category-tabs {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.category-tab {
  padding: 8px 16px;
  border-radius: var(--wave-radius-full);
  border: 1px solid var(--wave-border);
  background: transparent;
  color: var(--wave-muted);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s;

  &:hover {
    border-color: var(--wave-fg);
    color: var(--wave-fg);
  }

  &.active {
    background: var(--wave-accent);
    border-color: var(--wave-accent);
    color: #fff;
  }
}

.discover-content {
  padding: 32px 48px 48px;
  max-width: 960px;
  margin: 0 auto;
}

.content-section {
  margin-bottom: 48px;

  &:last-child {
    margin-bottom: 0;
  }
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 20px;
  color: var(--wave-fg);
}

.featured-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.room-card {
  background: var(--wave-surface);
  border: 1px solid var(--wave-border);
  border-radius: var(--wave-radius-lg);

  :deep(.el-card__body) {
    padding: 20px;
  }

  &.featured {
    .room-card-header {
      display: flex;
      align-items: flex-start;
      gap: 12px;
      margin-bottom: 12px;
    }

    .room-icon {
      width: 48px;
      height: 48px;
      border-radius: 12px;
      display: grid;
      place-items: center;
      font-size: 20px;
      font-weight: 700;
      color: #fff;
      flex-shrink: 0;
    }

    .room-info {
      flex: 1;
      min-width: 0;
    }

    .room-name {
      font-size: 16px;
      font-weight: 600;
      margin: 0 0 4px;
    }

    .room-meta {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 13px;
      color: var(--wave-muted);

      .el-icon {
        font-size: 14px;
      }
    }

    .room-description {
      font-size: 14px;
      color: var(--wave-muted);
      margin: 0 0 12px;
      line-height: 1.5;
    }

    .room-tags {
      display: flex;
      gap: 6px;
      flex-wrap: wrap;
      margin-bottom: 16px;

      .el-tag {
        background: var(--wave-bg);
        border-color: var(--wave-border);
        color: var(--wave-muted);
      }
    }

    .room-actions {
      display: flex;
      gap: 8px;
    }
  }
}

.trending-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.room-card.trending {
  :deep(.el-card__body) {
    padding: 16px 20px;
  }

  .trending-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .trending-info {
    flex: 1;
    min-width: 0;
  }

  .room-name {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 4px;
  }

  .room-description {
    font-size: 13px;
    color: var(--wave-muted);
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .room-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    color: var(--wave-muted);

    .el-icon {
      font-size: 14px;
    }

    .growth {
      color: var(--wave-accent-green);
      font-weight: 500;
    }
  }
}
</style>
