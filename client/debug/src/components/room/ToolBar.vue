<script setup lang="ts">
import { ref, computed, h } from 'vue'
import { useMessage } from 'naive-ui'
import { Wrench, Play, Settings2 } from 'lucide-vue-next'
import { allTools, getToolByKey, messageTestFunctions, type ToolDefinition, type ToolMenuItem, type ToolParam } from '@/utils/room'
import { useMultiUserAuthStore } from '@/stores/multiUserAuth'
import type { RoomMemberWithTestUser } from '@/utils/room/types'

const props = defineProps<{
  roomId: string
  /** 房间成员列表（包含测试用户信息） */
  roomMembers: RoomMemberWithTestUser[]
}>()

const emit = defineEmits<{
  (e: 'sendMessage', userId: string, content: string): void
}>()

const message = useMessage()
const multiUserAuthStore = useMultiUserAuthStore()

// ========== 状态 ==========
const selectedToolKey = ref<string>('')
const selectedMenuItemKey = ref<string>('')
const showParamModal = ref(false)
const paramValues = ref<Record<string, unknown>>({})
const executing = ref(false)

// ========== 计算属性 ==========
/** 当前激活的测试用户 */
const currentTestUser = computed(() => multiUserAuthStore.activeUser)

/** 有测试凭证的房间成员数量 */
const membersWithTestUserCount = computed(() =>
  props.roomMembers.filter(m => m.testUser).length
)

// ========== 计算属性 ==========
const selectedTool = computed(() => {
  return getToolByKey(selectedToolKey.value)
})

const selectedMenuItem = computed(() => {
  const tool = selectedTool.value
  if (!tool) return undefined
  return tool.menuItems.find(item => item.key === selectedMenuItemKey.value)
})

const toolOptions = computed(() => {
  return allTools.map(tool => ({
    label: tool.label,
    value: tool.key,
    icon: tool.icon,
  }))
})

const menuItemOptions = computed(() => {
  const tool = selectedTool.value
  if (!tool) return []
  return tool.menuItems.map(item => ({
    label: item.label,
    value: item.key,
    icon: item.icon,
  }))
})

// ========== 方法 ==========
const handleToolChange = (value: string) => {
  selectedToolKey.value = value
  selectedMenuItemKey.value = ''
  paramValues.value = {}
}

const handleMenuItemChange = (value: string) => {
  selectedMenuItemKey.value = value
  // 初始化参数默认值
  const menuItem = selectedMenuItem.value
  if (menuItem) {
    const defaults: Record<string, unknown> = {}
    for (const param of menuItem.params) {
      defaults[param.key] = param.defaultValue
    }
    paramValues.value = defaults
  }
}

const openParamModal = () => {
  if (!selectedMenuItem.value) {
    message.warning('请先选择一个工具操作')
    return
  }
  showParamModal.value = true
}

const handleExecute = async () => {
  if (!selectedTool.value || !selectedMenuItem.value) {
    message.warning('请先选择工具和操作')
    return
  }

  const toolKey = selectedTool.value.key
  const menuItemKey = selectedMenuItem.value.key

  executing.value = true

  try {
    const context = {
      roomId: props.roomId,
      roomMembers: props.roomMembers,
      currentTestUser: currentTestUser.value,
      sendMessage: async (userId: string, content: string) => {
        emit('sendMessage', userId, content)
      },
    }

    let result
    if (toolKey === 'messageTest') {
      const fn = messageTestFunctions[menuItemKey]
      if (fn) {
        result = await fn(context, paramValues.value)
      } else {
        throw new Error(`未知的操作: ${menuItemKey}`)
      }
    } else {
      throw new Error(`未知的工具: ${toolKey}`)
    }

    if (result.success) {
      message.success(result.message)
    } else {
      message.warning(result.message)
    }

    showParamModal.value = false
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    message.error(`执行失败: ${errorMsg}`)
  } finally {
    executing.value = false
  }
}

const renderParamInput = (param: ToolParam) => {
  const value = paramValues.value[param.key]

  switch (param.type) {
    case 'boolean':
      return h('div', {
        style: 'display: flex; align-items: center; gap: 8px; cursor: pointer;',
        onClick: () => {
          paramValues.value[param.key] = !value
        },
      }, [
        h('input', {
          type: 'checkbox',
          checked: value as boolean,
          onChange: (e: Event) => {
            paramValues.value[param.key] = (e.target as HTMLInputElement).checked
          },
        }),
        h('span', param.label),
      ])

    case 'select':
      return h('select', {
        value: value as string,
        onChange: (e: Event) => {
          paramValues.value[param.key] = (e.target as HTMLSelectElement).value
        },
        style: 'width: 100%; padding: 4px 8px; border: 1px solid var(--border-color); border-radius: var(--radius-sm);',
      }, param.options?.map(opt =>
        h('option', { value: opt.value }, opt.label)
      ))

    case 'number':
      return h('input', {
        type: 'number',
        value: value as number,
        min: param.min,
        max: param.max,
        step: param.step,
        onInput: (e: Event) => {
          const val = (e.target as HTMLInputElement).valueAsNumber
          paramValues.value[param.key] = isNaN(val) ? param.defaultValue : val
        },
        style: 'width: 100%; padding: 4px 8px; border: 1px solid var(--border-color); border-radius: var(--radius-sm);',
      })

    case 'string':
    default:
      return h('input', {
        type: 'text',
        value: value as string,
        onInput: (e: Event) => {
          paramValues.value[param.key] = (e.target as HTMLInputElement).value
        },
        style: 'width: 100%; padding: 4px 8px; border: 1px solid var(--border-color); border-radius: var(--radius-sm);',
      })
  }
}
</script>

<template>
  <div class="tool-bar">
    <n-space align="center" size="small">
      <!-- 工具选择 -->
      <n-select
        v-model:value="selectedToolKey"
        :options="toolOptions"
        placeholder="选择工具"
        style="width: 140px"
        size="small"
        @update:value="handleToolChange"
      >
        <template #render-option="{ node, option }">
          <n-space align="center" size="small">
            <component :is="option.icon" v-if="option.icon" class="icon-sm" />
            <span>{{ option.label }}</span>
          </n-space>
        </template>
      </n-select>

      <!-- 操作选择 -->
      <n-select
        v-model:value="selectedMenuItemKey"
        :options="menuItemOptions"
        placeholder="选择操作"
        style="width: 140px"
        size="small"
        :disabled="!selectedTool"
        @update:value="handleMenuItemChange"
      >
        <template #render-option="{ node, option }">
          <n-space align="center" size="small">
            <component :is="option.icon" v-if="option.icon" class="icon-sm" />
            <span>{{ option.label }}</span>
          </n-space>
        </template>
      </n-select>

      <!-- 执行按钮 -->
      <n-button
        type="primary"
        size="small"
        :disabled="!selectedMenuItem"
        @click="openParamModal"
      >
        <template #icon>
          <Play class="icon-sm" />
        </template>
        执行
      </n-button>

      <!-- 可操控成员数提示 -->
      <n-tag v-if="membersWithTestUserCount > 0" type="success" size="small">
        {{ membersWithTestUserCount }} 成员可操控
      </n-tag>
      <n-tag v-else type="warning" size="small">
        无成员可操控
      </n-tag>
    </n-space>

    <!-- 参数配置弹窗 -->
    <n-modal
      v-model:show="showParamModal"
      :title="selectedMenuItem?.label || '配置参数'"
      preset="dialog"
      positive-text="执行"
      negative-text="取消"
      :loading="executing"
      @positive-click="handleExecute"
    >
      <div v-if="selectedMenuItem" class="param-form">
        <p v-if="selectedMenuItem.description" class="param-description">
          {{ selectedMenuItem.description }}
        </p>
        <div
          v-for="param in selectedMenuItem.params"
          :key="param.key"
          class="param-item"
        >
          <label class="param-label">
            {{ param.label }}
            <n-tooltip v-if="param.description" trigger="hover">
              <template #trigger>
                <span class="param-help">?</span>
              </template>
              {{ param.description }}
            </n-tooltip>
          </label>
          <component :is="() => renderParamInput(param)" />
        </div>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.tool-bar {
  display: flex;
  align-items: center;
}

.param-form {
  padding: var(--space-md) 0;
}

.param-description {
  margin-bottom: var(--space-md);
  padding: var(--space-sm);
  background: var(--bg-secondary);
  border-radius: var(--radius-sm);
  font-size: 14px;
  color: var(--text-secondary);
}

.param-item {
  margin-bottom: var(--space-md);
}

.param-item:last-child {
  margin-bottom: 0;
}

.param-label {
  display: block;
  margin-bottom: var(--space-xs);
  font-size: 14px;
  font-weight: 500;
}

.param-help {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-left: var(--space-xs);
  background: var(--primary-color);
  color: white;
  border-radius: 50%;
  font-size: 10px;
  cursor: help;
}
</style>
