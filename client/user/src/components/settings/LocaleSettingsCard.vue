<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NCard,
  NSpace,
  NSelect,
  NList,
  NListItem,
  NThing,
  NButton,
  NDivider,
} from 'naive-ui'
import { Globe, Clock, Calendar } from 'lucide-vue-next'
import type { LocaleSettings, TimeFormat, DateFormat, WeekStartDay } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 语言与地区设置数据 */
  modelValue: LocaleSettings
  /** 保存中状态 */
  saving?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  saving: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新设置 */
  (e: 'update:modelValue', value: LocaleSettings): void
  /** 保存设置 */
  (e: 'save', value: LocaleSettings): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<LocaleSettings>({ ...props.modelValue })

/**
 * 监听外部数据变化
 */
watch(
  () => props.modelValue,
  (newVal) => {
    localSettings.value = { ...newVal }
  },
  { deep: true }
)

/**
 * 更新设置项
 */
function updateSetting<K extends keyof LocaleSettings>(
  key: K,
  value: LocaleSettings[K]
) {
  localSettings.value = { ...localSettings.value, [key]: value }
  emit('update:modelValue', localSettings.value)
}

/**
 * 保存设置
 */
function handleSave() {
  emit('save', localSettings.value)
}

/**
 * 语言选项
 */
const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: '繁體中文', value: 'zh-TW' },
  { label: 'English', value: 'en' },
  { label: '日本語', value: 'ja' },
]

/**
 * 时区选项（简化版）
 */
const timezoneOptions = [
  { label: '北京时间 (UTC+8)', value: 'Asia/Shanghai' },
  { label: '东京时间 (UTC+9)', value: 'Asia/Tokyo' },
  { label: '伦敦时间 (UTC+0)', value: 'Europe/London' },
  { label: '纽约时间 (UTC-5)', value: 'America/New_York' },
]

/**
 * 时间格式选项
 */
const timeFormatOptions: { label: string; value: TimeFormat }[] = [
  { label: '12小时制', value: '12h' },
  { label: '24小时制', value: '24h' },
]

/**
 * 日期格式选项
 */
const dateFormatOptions: { label: string; value: DateFormat }[] = [
  { label: 'YYYY-MM-DD', value: 'YYYY-MM-DD' },
  { label: 'DD/MM/YYYY', value: 'DD/MM/YYYY' },
  { label: 'MM/DD/YYYY', value: 'MM/DD/YYYY' },
]

/**
 * 星期起始日选项
 */
const weekStartOptions: { label: string; value: WeekStartDay }[] = [
  { label: '周一', value: 'monday' },
  { label: '周日', value: 'sunday' },
]
</script>

<template>
  <NCard title="语言与地区" class="settings-card">
    <NSpace vertical size="large">
      <!-- 语言设置 -->
      <NList bordered>
        <NListItem>
          <NThing title="界面语言" description="选择应用显示的语言">
            <template #avatar>
              <div class="setting-icon primary">
                <Globe :size="20" />
              </div>
            </template>
            <template #action>
              <NSelect
                :value="localSettings.language"
                :options="languageOptions"
                size="small"
                style="width: 140px"
                @update:value="(v) => updateSetting('language', v)"
              />
            </template>
          </NThing>
        </NListItem>
      </NList>

      <NDivider />

      <!-- 时区设置 -->
      <div class="settings-section">
        <h4 class="section-title">时区</h4>
        <NList bordered>
          <NListItem>
            <NThing title="时区" description="设置你的本地时区">
              <template #action>
                <NSelect
                  :value="localSettings.timezone"
                  :options="timezoneOptions"
                  size="small"
                  style="width: 180px"
                  @update:value="(v) => updateSetting('timezone', v)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <NDivider />

      <!-- 格式设置 -->
      <div class="settings-section">
        <h4 class="section-title">格式偏好</h4>
        <NList bordered>
          <NListItem>
            <NThing title="时间格式" description="选择时间显示格式">
              <template #avatar>
                <div class="setting-icon">
                  <Clock :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.timeFormat"
                  :options="timeFormatOptions"
                  size="small"
                  style="width: 120px"
                  @update:value="(v) => updateSetting('timeFormat', v as TimeFormat)"
                />
              </template>
            </NThing>
          </NListItem>
          <NListItem>
            <NThing title="日期格式" description="选择日期显示格式">
              <template #avatar>
                <div class="setting-icon">
                  <Calendar :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.dateFormat"
                  :options="dateFormatOptions"
                  size="small"
                  style="width: 140px"
                  @update:value="(v) => updateSetting('dateFormat', v as DateFormat)"
                />
              </template>
            </NThing>
          </NListItem>
          <NListItem>
            <NThing title="星期起始日" description="选择日历中每周的起始日">
              <template #action>
                <NSelect
                  :value="localSettings.weekStartDay"
                  :options="weekStartOptions"
                  size="small"
                  style="width: 100px"
                  @update:value="(v) => updateSetting('weekStartDay', v as WeekStartDay)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <!-- 保存按钮 -->
      <div class="card-actions">
        <NButton type="primary" :loading="saving" @click="handleSave">
          保存语言设置
        </NButton>
      </div>
    </NSpace>
  </NCard>
</template>

<style scoped>
.settings-card {
  margin-bottom: 16px;
}

.setting-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: var(--color-background-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.setting-icon.primary {
  background: var(--color-primary-soft);
  color: var(--color-primary);
}

.settings-section {
  margin-top: 8px;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}
</style>
