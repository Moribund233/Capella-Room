<script setup lang="ts">
import { ref, h, onMounted, computed } from 'vue'
import {
  NCard,
  NPagination,
  NButton,
  NSpace,
  NSwitch,
  NTooltip,
  NModal,
  useMessage,
  useDialog,
} from 'naive-ui'
import { ShieldCheck, Plus, Trash2, Eye, Shield } from 'lucide-vue-next'
import type { MobileAction } from '@/components/common/MobileTableCard.vue'
import { IPSearchForm, IPTable, IPDetailModal, IPAddModal } from '@/components/security'
import type { IPSearchParams } from '@/components/security/IPSearchForm.vue'
import type { IPAddFormData } from '@/components/security/IPAddModal.vue'
import { MobileTableCard } from '@/components/common'
import { useStatusBar } from '@/composables'
import { useAuthStore } from '@/store'
import {
  securityApi,
  type IPEntry,
  type IPListType,
  type WhitelistModeStatus,
} from '@/api/security'

const message = useMessage()
const dialog = useDialog()
const { setContent } = useStatusBar()
const authStore = useAuthStore()

// ==================== 数据状态 ====================

/** IP列表数据 */
const data = ref<IPEntry[]>([])
/** 加载状态 */
const loading = ref(false)
/** 白名单模式加载状态 */
const whitelistLoading = ref(false)
/** 总IP数量 */
const total = ref(0)
/** 当前页码 */
const page = ref(1)
/** 每页数量 */
const pageSize = ref(10)
/** 选中的IP keys */
const selectedKeys = ref<(string | number)[]>([])
/** 白名单模式状态 */
const whitelistMode = ref<WhitelistModeStatus | null>(null)

/** 搜索参数 */
const searchParams = ref<IPSearchParams>({
  ipAddress: '',
  listType: null,
})

/** 当前搜索参数缓存（用于刷新） */
const currentSearchParams = ref<IPSearchParams>({
  ipAddress: '',
  listType: null,
})

/** 添加IP弹窗显示状态 */
const showAddModal = ref(false)

/** 添加IP弹窗引用 */
const addModalRef = ref<InstanceType<typeof IPAddModal> | null>(null)

/** 是否为SuperAdmin */
const isSuperAdmin = computed(() => authStore.userInfo?.role === 'super_admin')

// ==================== 数据获取 ====================

/**
 * 获取IP列表
 * @param params 搜索参数
 */
const fetchIPList = async (params: {
  ipAddress?: string
  listType?: IPListType | null
  page?: number
  pageSize?: number
} = {}) => {
  loading.value = true

  try {
    const response = await securityApi.getIPList({
      page: params.page ?? page.value,
      page_size: params.pageSize ?? pageSize.value,
      ip_address: params.ipAddress || undefined,
      list_type: params.listType || undefined,
    })

    if (response.success && response.data) {
      data.value = response.data.items
      total.value = response.data.total
      page.value = params.page ?? page.value
      pageSize.value = params.pageSize ?? pageSize.value
      return true
    }
    return false
  } catch (error) {
    console.error('获取IP列表失败:', error)
    message.error('获取IP列表失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 获取白名单模式状态
 */
const fetchWhitelistMode = async () => {
  try {
    const response = await securityApi.getWhitelistMode()
    if (response.success && response.data) {
      whitelistMode.value = response.data
    }
  } catch (error) {
    console.error('获取白名单模式失败:', error)
  }
}

/**
 * 刷新当前列表
 */
const refresh = async () => {
  return fetchIPList({
    ipAddress: currentSearchParams.value.ipAddress,
    listType: currentSearchParams.value.listType,
    page: page.value,
    pageSize: pageSize.value,
  })
}

// ==================== 事件处理 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(ShieldCheck, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${total.value} 条IP规则`,
  ])
}

/**
 * 处理搜索
 */
const handleSearch = async (params: IPSearchParams) => {
  searchParams.value = params
  currentSearchParams.value = { ...params }
  selectedKeys.value = []

  const success = await fetchIPList({
    ipAddress: params.ipAddress,
    listType: params.listType,
    page: 1,
    pageSize: pageSize.value,
  })

  if (success) updateStatusBar()
}

/**
 * 重置搜索
 */
const handleReset = async () => {
  searchParams.value = {
    ipAddress: '',
    listType: null,
  }
  currentSearchParams.value = { ...searchParams.value }
  selectedKeys.value = []
  page.value = 1

  const success = await fetchIPList({ page: 1, pageSize: pageSize.value })

  if (success) {
    updateStatusBar()
    message.success('已重置筛选条件')
  }
}

/**
 * 刷新
 */
const handleRefresh = async () => {
  const success = await refresh()

  if (success) {
    updateStatusBar()
    message.success('刷新成功')
  }
}

/**
 * 分页变化
 */
const handlePageChange = async (newPage: number, newPageSize: number) => {
  selectedKeys.value = []

  const success = await fetchIPList({
    ipAddress: currentSearchParams.value.ipAddress,
    listType: currentSearchParams.value.listType,
    page: newPage,
    pageSize: newPageSize,
  })

  if (success) updateStatusBar()
}

/**
 * 查看IP详情
 */
const handleView = (ip: IPEntry) => {
  dialog.info({
    title: 'IP详情',
    content: () => h(IPDetailModal, { ip }),
    showIcon: false,
    closable: true,
    maskClosable: true,
    positiveText: '',
    style: {
      width: 'auto',
      maxWidth: 'calc(100vw - 32px)',
    },
  })
}

/**
 * 删除IP
 */
const handleDelete = async (ip: IPEntry) => {
  try {
    const response = await securityApi.deleteIP(ip.id)
    if (response.success) {
      message.success('IP已删除')
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '删除失败')
    }
  } catch {
    message.error('删除IP失败')
  }
}

/**
 * 批量删除
 */
const handleBatchDelete = async () => {
  if (selectedKeys.value.length === 0) {
    message.warning('请先选择要删除的IP')
    return
  }

  loading.value = true
  try {
    const response = await securityApi.batchDeleteIP(selectedKeys.value as string[])
    if (response.success) {
      message.success(`已成功删除 ${selectedKeys.value.length} 条IP规则`)
      selectedKeys.value = []
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '批量删除失败')
    }
  } catch {
    message.error('批量删除失败')
  } finally {
    loading.value = false
  }
}

/**
 * 打开添加IP弹窗
 */
const handleOpenAddModal = () => {
  showAddModal.value = true
}

/**
 * 关闭添加IP弹窗
 */
const handleCloseAddModal = () => {
  showAddModal.value = false
}

/**
 * 处理添加IP弹窗确认
 */
const handleAddModalConfirm = () => {
  addModalRef.value?.handleConfirm()
}

/**
 * 确认添加IP
 */
const handleConfirmAdd = async (formData: IPAddFormData) => {
  try {
    const response = await securityApi.addIP({
      ip_address: formData.ipAddress,
      list_type: formData.listType,
      remark: formData.remark,
      expires_at: formData.expireTime ? new Date(formData.expireTime).toISOString() : null,
    })

    if (response.success) {
      message.success('IP添加成功')
      showAddModal.value = false
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '添加失败')
    }
  } catch {
    message.error('添加 IP 失败')
  }
}

/**
 * 切换白名单模式
 */
const handleWhitelistModeChange = async (value: boolean) => {
  if (!isSuperAdmin.value) {
    message.error('只有超级管理员可以修改白名单模式')
    return
  }

  whitelistLoading.value = true
  try {
    const response = await securityApi.setWhitelistMode({ enabled: value })
    if (response.success && response.data) {
      whitelistMode.value = response.data
      message.success(value ? '白名单模式已启用' : '白名单模式已关闭')
    } else {
      message.error(response.message || '设置失败')
    }
  } catch {
    message.error('设置白名单模式失败')
  } finally {
    whitelistLoading.value = false
  }
}

/**
 * 处理移动端行点击
 */
const handleMobileRowClick = (row: unknown) => {
  handleView(row as IPEntry)
}

// ==================== 移动端配置 ====================

/** 移动端列配置 */
const mobileColumns = [
  { key: 'list_type', title: '类型' },
  { key: 'remark', title: '备注' },
  { key: 'expires_at', title: '过期时间' },
]

/** 移动端操作按钮配置 */
const mobileActions: MobileAction<IPEntry>[] = [
  {
    label: '查看',
    icon: Eye,
    type: 'default',
    onClick: (ip: IPEntry) => handleView(ip),
  },
  {
    label: '删除',
    icon: Trash2,
    type: 'error',
    onClick: (ip: IPEntry) => {
      dialog.warning({
        title: '确认删除',
        content: `确定要删除IP "${ip.ip_address}" 吗？`,
        positiveText: '删除',
        negativeText: '取消',
        onPositiveClick: () => handleDelete(ip),
      })
    },
  },
]

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchIPList({ page: 1, pageSize: 10 })
  if (success) updateStatusBar()
  await fetchWhitelistMode()
})
</script>

<template>
  <div class="ip-security-view">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">IP安全管理</h1>
        <p class="page-description">管理IP黑白名单，保护系统安全</p>
      </div>
      <div class="header-right">
        <NTooltip placement="left">
          <template #trigger>
            <div class="whitelist-toggle">
              <span class="toggle-label">白名单模式</span>
              <NSwitch
                :value="whitelistMode?.enabled"
                :loading="whitelistLoading"
                :disabled="!isSuperAdmin"
                @update:value="handleWhitelistModeChange"
              >
                <template #checked>
                  <Shield :size="12" />
                </template>
                <template #unchecked>
                  <ShieldCheck :size="12" />
                </template>
              </NSwitch>
            </div>
          </template>
          <span>
            {{ isSuperAdmin ? '启用后只允许白名单IP访问' : '只有超级管理员可以修改' }}
          </span>
        </NTooltip>
      </div>
    </div>

    <NCard class="search-card" :bordered="false">
      <IPSearchForm
        v-bind="searchParams"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
      />
    </NCard>

    <NCard class="toolbar-card" :bordered="false">
      <div class="toolbar-content">
        <div class="toolbar-left">
          <span v-if="selectedKeys.length > 0" class="selected-info">
            已选择 {{ selectedKeys.length }} 条IP规则
          </span>
        </div>
        <div class="toolbar-right">
          <NSpace>
            <NButton
              v-if="selectedKeys.length > 0"
              type="error"
              size="small"
              :loading="loading"
              @click="handleBatchDelete"
            >
              <template #icon>
                <Trash2 :size="16" />
              </template>
              批量删除
            </NButton>
            <NButton type="primary" size="small" @click="handleOpenAddModal">
              <template #icon>
                <Plus :size="16" />
              </template>
              添加 IP
            </NButton>
          </NSpace>
        </div>
      </div>
    </NCard>

    <NCard class="table-card" :bordered="false">
      <!-- 桌面端：IPTable 组件 -->
      <div class="desktop-view">
        <IPTable
          v-model:selected-keys="selectedKeys"
          :data="data"
          :loading="loading"
          @view="handleView"
          @delete="handleDelete"
        />
        <div v-if="total > 0" class="pagination-wrapper">
          <NPagination
            :page="page"
            :page-size="pageSize"
            :item-count="total"
            :page-sizes="[10, 20, 50, 100]"
            show-size-picker
            show-quick-jumper
            @update:page="handlePageChange($event, pageSize)"
            @update:page-size="handlePageChange(1, $event)"
          />
        </div>
      </div>

      <!-- 移动端：卡片视图 -->
      <div class="mobile-view">
        <MobileTableCard
          :data="data"
          :columns="mobileColumns"
          title-column="ip_address"
          :actions="mobileActions as MobileAction[]"
          @row-click="handleMobileRowClick"
        />
        <div v-if="total > 0" class="mobile-pagination">
          <NPagination
            :page="page"
            :page-count="Math.ceil(total / pageSize)"
            :simple="true"
            @update:page="handlePageChange($event, pageSize)"
          />
        </div>
      </div>
    </NCard>

    <!-- 添加IP弹窗 -->
    <NModal
      v-model:show="showAddModal"
      title="添加 IP"
      preset="dialog"
      positive-text="确认"
      negative-text="取消"
      @positive-click="handleAddModalConfirm"
      @negative-click="handleCloseAddModal"
      @close="handleCloseAddModal"
      style="width: 500px; max-width: calc(100vw - 32px)"
    >
      <IPAddModal ref="addModalRef" @confirm="handleConfirmAdd" @cancel="handleCloseAddModal" />
    </NModal>
  </div>
</template>

<style scoped>
.ip-security-view {
  padding: 24px;
  min-height: 100%;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-left {
  flex: 1;
}

.header-right {
  display: flex;
  align-items: center;
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.page-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.whitelist-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
}

.toggle-label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.search-card {
  margin-bottom: 16px;
}

.toolbar-card {
  margin-bottom: 16px;
}

.toolbar-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selected-info {
  font-size: 14px;
  color: var(--text-secondary);
}

.table-card {
  margin-bottom: 24px;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.mobile-pagination {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}

/* 响应式布局 */
.desktop-view {
  display: block;
}

.mobile-view {
  display: none;
}

@media (max-width: 768px) {
  .ip-security-view {
    padding: 16px;
  }

  .page-header {
    flex-direction: column;
    gap: 16px;
  }

  .page-title {
    font-size: 24px;
  }

  .whitelist-toggle {
    width: 100%;
    justify-content: space-between;
  }

  .desktop-view {
    display: none;
  }

  .mobile-view {
    display: block;
  }

  .toolbar-content {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
}
</style>
