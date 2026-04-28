<script setup lang="ts">
import { computed, onMounted, ref, h } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  NCard, NGrid, NGridItem, NStatistic, NButton, NTag, NAvatar,
  NEmpty, NList, NListItem, NThing, NText, NDivider, NSpace, NBadge,
  NInput, NPagination, NDropdown, useDialog, useMessage
} from 'naive-ui'
import { ChartCard, PieChart, BarChart } from '@/components/common/charts'
import type { PieDataItem, BarSeries } from '@/components/common/charts'
import type { MemberInfo } from '@/api/rooms'
import {
  ArrowLeft, Activity, Shield, Clock, Hash, Lock, Globe, MoreVertical
} from 'lucide-vue-next'
import { useRoomStore } from '@/stores'

const route = useRoute()
const router = useRouter()

/**
 * 使用房间管理 Store
 */
const roomStore = useRoomStore()

// 使用 storeToRefs 保持响应性
const {
  currentRoom,
  members,
  membersLoading,
  messagesTotal,
} = storeToRefs(roomStore)



/** 当前房间ID */
const roomId = computed(() => route.params.id as string)

/** 房间名称 */
const roomName = computed(() => currentRoom.value?.name || '未知房间')

/** 房间饱和度 */
const roomSaturation = computed(() => {
  if (!currentRoom.value) return 0
  return Math.round((currentRoom.value.member_count / currentRoom.value.max_members) * 100)
})

/** 在线成员数 */
const onlineMembers = computed(() => {
  return members.value.filter(m => m.user_status === 'online').length
})

/** 成员在线率 */
const onlineRate = computed(() => {
  if (members.value.length === 0) return 0
  return Math.round((onlineMembers.value / members.value.length) * 100)
})

/** 人均消息数 */
const messagesPerMember = computed(() => {
  if (!currentRoom.value || currentRoom.value.member_count === 0) return 0
  return Math.round(messagesTotal.value / currentRoom.value.member_count)
})

/** 成员角色分布饼图数据 */
const memberRolePieData = computed<PieDataItem[]>(() => {
  const stats = { owner: 0, admin: 0, member: 0 }
  members.value.forEach((m) => {
    stats[m.role]++
  })
  return [
    { name: '房主', value: stats.owner, itemStyle: { color: '#f5222d' } },
    { name: '管理员', value: stats.admin, itemStyle: { color: '#faad14' } },
    { name: '成员', value: stats.member, itemStyle: { color: '#1890ff' } },
  ].filter(item => item.value > 0)
})

/** 成员在线状态分布 */
const memberStatusPieData = computed<PieDataItem[]>(() => {
  const online = members.value.filter(m => m.user_status === 'online').length
  const offline = members.value.filter(m => m.user_status === 'offline').length
  const away = members.value.filter(m => m.user_status === 'away').length
  return [
    { name: '在线', value: online, itemStyle: { color: '#52c41a' } },
    { name: '离线', value: offline, itemStyle: { color: '#8c8c8c' } },
    { name: '离开', value: away, itemStyle: { color: '#faad14' } },
  ].filter(item => item.value > 0)
})

/** 消息类型分布 X 轴 */
const messageTypeXAxis = ['文本', '图片', '文件', '系统']

/** 消息类型分布（模拟数据，实际应从API获取） */
const messageTypeBarData = computed<BarSeries[]>(() => {
  const total = messagesTotal.value || 0
  return [{
    name: '消息数量',
    data: [
      Math.round(total * 0.85),
      Math.round(total * 0.1),
      Math.round(total * 0.03),
      Math.round(total * 0.02),
    ],
    itemStyle: {
      color: (params: { dataIndex: number }) => {
        const colors = ['#1890ff', '#52c41a', '#faad14', '#8c8c8c']
        return colors[params.dataIndex] || '#1890ff'
      }
    }
  }]
})

/** 计算创建天数 */
const getCreatedDays = (createdAt: string) => {
  const created = new Date(createdAt)
  const now = new Date()
  const diff = now.getTime() - created.getTime()
  return Math.floor(diff / (1000 * 60 * 60 * 24))
}

/** 格式化时间 */
const formatTime = (time: string) => {
  return new Date(time).toLocaleString('zh-CN')
}

/** 格式化日期 */
const formatDate = (time: string) => {
  return new Date(time).toLocaleDateString('zh-CN')
}

/** 返回上一页 */
const handleBack = () => {
  router.back()
}

// ========== 成员列表分页和搜索 ==========
const memberSearchQuery = ref('')
const currentPage = ref(1)
const pageSize = 10

/** 过滤后的成员列表 */
const filteredMembers = computed(() => {
  if (!memberSearchQuery.value) return members.value
  const query = memberSearchQuery.value.toLowerCase()
  return members.value.filter(m =>
    m.username.toLowerCase().includes(query) ||
    m.email.toLowerCase().includes(query)
  )
})

/** 分页后的成员列表 */
const paginatedMembers = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  const end = start + pageSize
  return filteredMembers.value.slice(start, end)
})

// ========== 成员操作 ==========
const dialog = useDialog()
const message = useMessage()

/** 是否可以管理该成员（路由守卫已确保只有 admin+ 能进入此页面） */
const canManageMember = () => true

/** 获取成员操作选项 */
const getMemberActions = (member: MemberInfo) => {
  const actions = []

  // 设置角色选项
  if (member.role === 'member') {
    actions.push({
      label: '设为房间管理员',
      key: 'set_admin',
      icon: () => h(Shield, { size: 14 })
    })
  } else if (member.role === 'admin') {
    actions.push({
      label: '设为普通成员',
      key: 'set_member'
    })
  } else if (member.role === 'owner') {
    // 甚至可以转移房主权限（谨慎操作）
    actions.push({
      label: '转让房主权限（谨慎）',
      key: 'transfer_owner',
      icon: () => h('span', { style: 'color: #faad14' }, '⚠️'),
      props: {
        style: 'color: #faad14'
      }
    })
  }

  // 踢出成员
  actions.push({
    label: '踢出房间',
    key: 'kick',
    icon: () => h('span', { style: 'color: #f5222d' }, '🚫'),
    props: {
      style: 'color: #f5222d'
    }
  })

  return actions
}

/** 处理成员操作 */
const handleMemberAction = async (key: string, member: MemberInfo) => {
  if (!roomId.value) return

  switch (key) {
    case 'set_admin':
    case 'set_member':
      dialog.warning({
        title: '确认修改角色',
        content: `确定要将 ${member.username} 设为${key === 'set_admin' ? '房间管理员' : '普通成员'}吗？`,
        positiveText: '确认',
        negativeText: '取消',
        onPositiveClick: async () => {
          try {
            await roomStore.setMemberRole(roomId.value, member.user_id, key === 'set_admin' ? 'admin' : 'member')
            message.success('角色修改成功')
            await roomStore.fetchMembers(roomId.value)
          } catch {
            message.error('角色修改失败')
          }
        }
      })
      break
    case 'transfer_owner':
      dialog.error({
        title: '⚠️ 危险操作：转让房主权限',
        content: `确定要将房主权限转让给 ${member.username} 吗？转让后您将失去该房间的管理权限！`,
        positiveText: '确认转让',
        negativeText: '取消',
        onPositiveClick: async () => {
          try {
            await roomStore.setMemberRole(roomId.value, member.user_id, 'owner')
            message.success('房主权限已转让')
            await roomStore.fetchMembers(roomId.value)
          } catch {
            message.error('操作失败')
          }
        }
      })
      break
    case 'kick':
      dialog.error({
        title: '确认踢出成员',
        content: `确定要将 ${member.username} 踢出房间吗？此操作不可恢复。`,
        positiveText: '确认踢出',
        negativeText: '取消',
        onPositiveClick: async () => {
          try {
            await roomStore.kickMember(roomId.value, member.user_id)
            message.success('成员已踢出')
            await roomStore.fetchMembers(roomId.value)
          } catch {
            message.error('操作失败')
          }
        }
      })
      break
  }
}

// 页面加载时获取数据
onMounted(async () => {
  if (roomId.value) {
    if (!currentRoom.value || currentRoom.value.id !== roomId.value) {
      await roomStore.fetchRoomDetail(roomId.value)
    }
    await roomStore.fetchMembers(roomId.value)
    // 加载消息统计数据
    await roomStore.fetchRoomMessages(roomId.value, { page: 1, pageSize: 1 })
  }
})
</script>

<template>
  <div class="room-analytics-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <NButton quaternary circle @click="handleBack">
          <template #icon>
            <ArrowLeft :size="20" />
          </template>
        </NButton>
        <div class="header-title">
          <h1 class="page-title">房间数据分析</h1>
          <p class="page-subtitle">
            <NTag size="small" :type="currentRoom?.is_private ? 'warning' : 'success'">
              <template #icon>
                <component :is="currentRoom?.is_private ? Lock : Globe" :size="12" />
              </template>
              {{ currentRoom?.is_private ? '私有' : '公开' }}
            </NTag>
            <span class="room-name">{{ roomName }}</span>
          </p>
        </div>
      </div>
    </div>

    <div v-if="currentRoom" class="analytics-content">
      <!-- 核心指标卡片 - 紧凑布局 -->
      <NGrid :cols="6" :x-gap="8" :y-gap="8" responsive="screen">
        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="成员" :value="currentRoom.member_count">
              <template #suffix>
                <span class="stat-suffix">/ {{ currentRoom.max_members }}</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NBadge :value="`${roomSaturation}%`" :type="roomSaturation > 80 ? 'error' : roomSaturation > 50 ? 'warning' : 'success'" />
            </div>
          </NCard>
        </NGridItem>

        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="在线" :value="onlineMembers">
              <template #suffix>
                <span class="stat-suffix">({{ onlineRate }}%)</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NTag size="tiny" :type="onlineRate > 50 ? 'success' : onlineRate > 20 ? 'warning' : 'default'">
                {{ onlineRate > 50 ? '活跃' : onlineRate > 20 ? '一般' : '低' }}
              </NTag>
            </div>
          </NCard>
        </NGridItem>

        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="消息" :value="messagesTotal">
              <template #suffix>
                <span class="stat-suffix">({{ messagesPerMember }}/人)</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NTag size="tiny" type="info">总计</NTag>
            </div>
          </NCard>
        </NGridItem>

        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="运营" :value="getCreatedDays(currentRoom.created_at)">
              <template #suffix>
                <span class="stat-suffix">天</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NTag size="tiny" type="default">{{ formatDate(currentRoom.created_at).slice(5) }}</NTag>
            </div>
          </NCard>
        </NGridItem>

        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="管理员" :value="memberRolePieData.find(i => i.name === '管理员')?.value || 0">
              <template #suffix>
                <span class="stat-suffix">人</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NTag size="tiny" type="warning">团队</NTag>
            </div>
          </NCard>
        </NGridItem>

        <NGridItem span="3 s:2 l:1">
          <NCard class="stat-card mini" size="small">
            <NStatistic label="评分" :value="Math.min(100, messagesPerMember * 2 + onlineRate)">
              <template #suffix>
                <span class="stat-suffix">分</span>
              </template>
            </NStatistic>
            <div class="stat-badge">
              <NTag size="tiny" :type="messagesPerMember > 50 ? 'success' : messagesPerMember > 20 ? 'warning' : 'default'">
                {{ messagesPerMember > 50 ? '活跃' : messagesPerMember > 20 ? '一般' : '冷清' }}
              </NTag>
            </div>
          </NCard>
        </NGridItem>
      </NGrid>

      <!-- 房间信息 + 图表区域 -->
      <NGrid :cols="3" :x-gap="16" :y-gap="16" responsive="screen">
        <!-- 房间基本信息 - 紧凑卡片 -->
        <NGridItem span="3 s:3 l:1">
          <NCard title="房间信息" size="small" class="info-card">
            <NSpace vertical :size="12">
              <div class="info-row">
                <span class="info-label">
                  <Hash :size="14" /> 房间ID
                </span>
                <NText code class="info-value">{{ currentRoom.id }}</NText>
              </div>
              <NDivider class="compact-divider" />
              <div class="info-row">
                <span class="info-label">
                  <Clock :size="14" /> 创建时间
                </span>
                <NText class="info-value">{{ formatTime(currentRoom.created_at) }}</NText>
              </div>
              <NDivider class="compact-divider" />
              <div class="info-row">
                <span class="info-label">
                  <Activity :size="14" /> 最后更新
                </span>
                <NText class="info-value">{{ formatTime(currentRoom.updated_at) }}</NText>
              </div>
              <NDivider class="compact-divider" />
              <div class="info-row">
                <span class="info-label">
                  <Shield :size="14" /> 房主
                </span>
                <NSpace align="center" :size="8">
                  <NAvatar
                    v-if="currentRoom.owner?.avatar_url"
                    :src="currentRoom.owner.avatar_url"
                    :size="20"
                    round
                  />
                  <NAvatar v-else :size="20" round>
                    {{ currentRoom.owner?.username?.charAt(0).toUpperCase() || '?' }}
                  </NAvatar>
                  <NText strong>{{ currentRoom.owner?.username || '未知' }}</NText>
                </NSpace>
              </div>
              <NDivider class="compact-divider" />
              <div class="info-row">
                <span class="info-label">房间描述</span>
                <NText depth="3" class="info-value description">
                  {{ currentRoom.description || '暂无描述' }}
                </NText>
              </div>
            </NSpace>
          </NCard>
        </NGridItem>

        <!-- 图表区域 -->
        <NGridItem span="3 s:3 l:2">
          <NGrid :cols="2" :x-gap="12" :y-gap="12">
            <NGridItem span="2 s:1">
              <ChartCard title="角色分布" subtitle="成员权限统计" min-height="200px">
                <PieChart
                  :data="memberRolePieData"
                  type="doughnut"
                  :show-label="true"
                  :show-legend="true"
                  legend-position="bottom"
                  :loading="membersLoading"
                />
              </ChartCard>
            </NGridItem>
            <NGridItem span="2 s:1">
              <ChartCard title="在线状态" subtitle="成员活跃度" min-height="200px">
                <PieChart
                  :data="memberStatusPieData"
                  type="pie"
                  :show-label="true"
                  :show-legend="true"
                  legend-position="bottom"
                  :loading="membersLoading"
                />
              </ChartCard>
            </NGridItem>
            <NGridItem span="2">
              <ChartCard title="消息类型分布" subtitle="内容构成分析" min-height="180px">
                <BarChart
                  :x-axis="messageTypeXAxis"
                  :series="messageTypeBarData"
                  :show-label="true"
                  :loading="membersLoading"
                />
              </ChartCard>
            </NGridItem>
          </NGrid>
        </NGridItem>
      </NGrid>

      <!-- 成员列表 -->
      <NCard title="成员列表" size="small" :bordered="false" :loading="membersLoading" class="members-card">
        <template #header-extra>
          <NSpace align="center" :size="12">
            <NInput
              v-model:value="memberSearchQuery"
              placeholder="搜索成员"
              size="small"
              clearable
              style="width: 150px"
            />
            <NTag size="small" type="info">共 {{ filteredMembers.length }} 人</NTag>
          </NSpace>
        </template>
        <NEmpty v-if="filteredMembers.length === 0" description="暂无成员数据" />
        <NList v-else hoverable size="small">
          <NListItem v-for="member in paginatedMembers" :key="member.user_id">
            <NThing>
              <template #avatar>
                <NBadge
                  :dot="true"
                  :type="member.user_status === 'online' ? 'success' : member.user_status === 'away' ? 'warning' : 'default'"
                  :offset="[-2, 28]"
                >
                  <NAvatar
                    v-if="member.avatar_url"
                    :src="member.avatar_url"
                    :size="32"
                    round
                  />
                  <NAvatar v-else :size="32" round>
                    {{ member.username.charAt(0).toUpperCase() }}
                  </NAvatar>
                </NBadge>
              </template>
              <template #header>
                <NSpace align="center" :size="8">
                  <NText strong>{{ member.username }}</NText>
                  <NTag
                    size="tiny"
                    :type="member.role === 'owner' ? 'error' : member.role === 'admin' ? 'warning' : 'default'"
                  >
                    {{ member.role === 'owner' ? '房主' : member.role === 'admin' ? '管理员' : '成员' }}
                  </NTag>
                </NSpace>
              </template>
              <template #header-extra>
                <NSpace align="center" :size="8">
                  <NText depth="3" style="font-size: 12px">
                    {{ member.user_status === 'online' ? '🟢 在线' : member.user_status === 'away' ? '🟡 离开' : '⚪ 离线' }}
                  </NText>
                  <!-- 操作按钮 -->
                  <NDropdown
                    v-if="canManageMember()"
                    :options="getMemberActions(member)"
                    @select="(key) => handleMemberAction(key, member)"
                  >
                    <NButton size="tiny" quaternary>
                      <template #icon>
                        <MoreVertical :size="14" />
                      </template>
                    </NButton>
                  </NDropdown>
                </NSpace>
              </template>
              <template #description>
                <NText depth="3" style="font-size: 12px">
                  加入于 {{ formatDate(member.joined_at) }} · {{ member.email }}
                </NText>
              </template>
            </NThing>
          </NListItem>
        </NList>
        <!-- 分页 -->
        <div v-if="filteredMembers.length > pageSize" class="pagination-wrapper">
          <NPagination
            v-model:page="currentPage"
            :page-size="pageSize"
            :item-count="filteredMembers.length"
            size="small"
          />
        </div>
      </NCard>
    </div>

    <div v-else class="loading-state">
      <NEmpty description="加载中..." />
    </div>
  </div>
</template>

<style scoped>
.room-analytics-page {
  min-height: 100%;
}

.page-header {
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
  line-height: 1.2;
}

.page-subtitle {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 14px;
}

.room-name {
  color: var(--text-secondary);
}

.analytics-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 统计卡片紧凑样式 */
.stat-card :deep(.n-card__content) {
  padding: 12px 16px;
}

.stat-card.mini :deep(.n-card__content) {
  padding: 8px 12px;
}

.stat-card :deep(.n-statistic__label) {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.stat-card.mini :deep(.n-statistic__label) {
  font-size: 11px;
  margin-bottom: 2px;
}

.stat-card :deep(.n-statistic__value) {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.stat-card.mini :deep(.n-statistic__value) {
  font-size: 20px;
  gap: 4px;
}

.stat-suffix {
  font-size: 12px;
  font-weight: 400;
  color: var(--text-secondary);
}

.stat-card.mini .stat-suffix {
  font-size: 11px;
}

.stat-badge {
  margin-top: 8px;
}

.stat-card.mini .stat-badge {
  margin-top: 4px;
}

/* 信息卡片紧凑样式 */
.info-card :deep(.n-card__content) {
  padding: 12px 16px;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.info-value {
  font-size: 13px;
  color: var(--text-primary);
}

.info-value.description {
  line-height: 1.5;
}

.compact-divider {
  margin: 8px 0;
}

/* 成员列表样式 */
.members-card :deep(.n-card__content) {
  padding: 0;
}

.members-card :deep(.n-list) {
  --n-padding-left: 16px;
  --n-padding-right: 16px;
}

.pagination-wrapper {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.loading-state {
  text-align: center;
  padding: 48px 0;
}

/* 移动端适配 */
@media screen and (max-width: 768px) {
  .page-title {
    font-size: 20px;
  }

  .stat-card :deep(.n-statistic__value) {
    font-size: 20px;
  }
}
</style>
