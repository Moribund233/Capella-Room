<script setup lang="ts">
import { ref } from 'vue'
import { useMessage } from 'naive-ui'
import { Send, Play, Trash2, Save, FileJson, Clock, CheckCircle, XCircle } from 'lucide-vue-next'
import { apiClient } from '@/api'

const apiBaseUrl = ref('http://localhost:8080/api/v1')
const selectedMethod = ref('GET')
const apiPath = ref('/rooms')
const requestBody = ref('')
const responseData = ref('')
const responseStatus = ref<number | null>(null)
const responseTime = ref(0)

const methods = [
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'PATCH', value: 'PATCH' }
]

const apiEndpoints = [
  { label: '获取房间列表', method: 'GET', path: '/rooms', category: '房间' },
  { label: '创建房间', method: 'POST', path: '/rooms', category: '房间' },
  { label: '获取房间详情', method: 'GET', path: '/rooms/:id', category: '房间' },
  { label: '更新房间', method: 'PUT', path: '/rooms/:id', category: '房间' },
  { label: '删除房间', method: 'DELETE', path: '/rooms/:id', category: '房间' },
  { label: '用户注册', method: 'POST', path: '/auth/register', category: '认证' },
  { label: '用户登录', method: 'POST', path: '/auth/login', category: '认证' },
  { label: '刷新Token', method: 'POST', path: '/auth/refresh', category: '认证' },
  { label: '获取当前用户', method: 'GET', path: '/users/me', category: '用户' },
  { label: '获取用户列表', method: 'GET', path: '/users', category: '用户' },
  { label: '获取房间消息', method: 'GET', path: '/messages/:room_id', category: '消息' },
  { label: '搜索消息', method: 'GET', path: '/messages/search', category: '消息' }
]

const requestHistory = ref([
  { method: 'GET', path: '/api/v1/rooms', status: 200, time: '45ms', timestamp: '10:30:15' },
  { method: 'POST', path: '/api/v1/auth/login', status: 200, time: '120ms', timestamp: '10:28:32' },
  { method: 'GET', path: '/api/v1/users/me', status: 401, time: '15ms', timestamp: '10:25:10' }
])

const message = useMessage()

const selectEndpoint = (endpoint: (typeof apiEndpoints)[0]) => {
  selectedMethod.value = endpoint.method
  apiPath.value = endpoint.path
}

const sendRequest = async () => {
  const startTime = Date.now()
  const url = `${apiBaseUrl.value}${apiPath.value}`

  try {
    let response: any
    const endpoint = apiPath.value
    const body = requestBody.value ? JSON.parse(requestBody.value) : undefined

    switch (selectedMethod.value) {
      case 'GET':
        response = await apiClient.get(endpoint)
        break
      case 'POST':
        response = await apiClient.post(endpoint, body)
        break
      case 'PUT':
        response = await apiClient.put(endpoint, body)
        break
      case 'DELETE':
        response = await apiClient.delete(endpoint)
        break
      case 'PATCH':
        response = await apiClient.patch(endpoint, body)
        break
      default:
        throw new Error(`Unsupported method: ${selectedMethod.value}`)
    }

    responseStatus.value = 200
    responseTime.value = Date.now() - startTime
    responseData.value = JSON.stringify(response, null, 2)

    // 添加到历史记录
    requestHistory.value.unshift({
      method: selectedMethod.value,
      path: apiPath.value,
      status: 200,
      time: `${responseTime.value}ms`,
      timestamp: new Date().toLocaleTimeString()
    })
  } catch (error) {
    responseStatus.value = 500
    responseTime.value = Date.now() - startTime
    responseData.value = JSON.stringify({
      error: error instanceof Error ? error.message : 'Unknown error'
    }, null, 2)

    requestHistory.value.unshift({
      method: selectedMethod.value,
      path: apiPath.value,
      status: 500,
      time: `${responseTime.value}ms`,
      timestamp: new Date().toLocaleTimeString()
    })

    message.error(`请求失败: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

const clearResponse = () => {
  responseData.value = ''
  responseStatus.value = null
  responseTime.value = 0
}

const loadTemplate = (method: string) => {
  const templates: Record<string, string> = {
    POST: JSON.stringify({ name: '新房间', description: '房间描述', is_public: true }, null, 2),
    PUT: JSON.stringify({ name: '更新的房间', description: '更新后的描述' }, null, 2),   PATCH: JSON.stringify({ description: '部分更新' }, null, 2)
  }
  requestBody.value = templates[method] || ''
}
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <FileJson class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        API 测试
      </h1>
      <p class="page-subtitle">测试 REST API 接口</p>
    </div>

    <div style="display: grid; grid-template-columns: 280px 1fr; gap: var(--space-lg)">
      <!-- 左侧：API 列表 -->
      <n-card title="API 端点" style="max-height: 600px; overflow-y: auto">
        <n-collapse>
          <n-collapse-item title="认证" name="auth">
            <n-list hoverable clickable>
              <n-list-item
                v-for="ep in apiEndpoints.filter((e) => e.category === '认证')"
                :key="ep.path"
                @click="selectEndpoint(ep)"
              >
                <n-space align="center">
                  <n-tag :type="ep.method === 'GET' ? 'success' : 'info'" size="small">
                    {{ ep.method }}
                  </n-tag>
                  <span style="font-size: 13px">{{ ep.label }}</span>
                </n-space>
              </n-list-item>
            </n-list>
          </n-collapse-item>
          <n-collapse-item title="房间" name="rooms">
            <n-list hoverable clickable>
              <n-list-item
                v-for="ep in apiEndpoints.filter((e) => e.category === '房间')"
                :key="ep.path"
                @click="selectEndpoint(ep)"
              >
                <n-space align="center">
                  <n-tag
                    :type="
                      ep.method === 'GET'
                        ? 'success'
                        : ep.method === 'POST'
                          ? 'info'
                          : ep.method === 'PUT'
                            ? 'warning'
                            : 'error'
                    "
                    size="small"
                  >
                    {{ ep.method }}
                  </n-tag>
                  <span style="font-size: 13px">{{ ep.label }}</span>
                </n-space>
              </n-list-item>
            </n-list>
          </n-collapse-item>
          <n-collapse-item title="用户" name="users">
            <n-list hoverable clickable>
              <n-list-item
                v-for="ep in apiEndpoints.filter((e) => e.category === '用户')"
                :key="ep.path"
                @click="selectEndpoint(ep)"
              >
                <n-space align="center">
                  <n-tag :type="ep.method === 'GET' ? 'success' : 'info'" size="small">
                    {{ ep.method }}
                  </n-tag>
                  <span style="font-size: 13px">{{ ep.label }}</span>
                </n-space>
              </n-list-item>
            </n-list>
          </n-collapse-item>
          <n-collapse-item title="消息" name="messages">
            <n-list hoverable clickable>
              <n-list-item
                v-for="ep in apiEndpoints.filter((e) => e.category === '消息')"
                :key="ep.path"
                @click="selectEndpoint(ep)"
              >
                <n-space align="center">
                  <n-tag :type="ep.method === 'GET' ? 'success' : 'info'" size="small">
                    {{ ep.method }}
                  </n-tag>
                  <span style="font-size: 13px">{{ ep.label }}</span>
                </n-space>
              </n-list-item>
            </n-list>
          </n-collapse-item>
        </n-collapse>
      </n-card>

      <!-- 右侧：请求测试 -->
      <div style="display: flex; flex-direction: column; gap: var(--space-lg)">
        <n-card title="请求配置">
          <n-space vertical size="large">
            <n-input-group>
              <n-select
                v-model:value="selectedMethod"
                :options="methods"
                style="width: 120px"
                @update:value="loadTemplate"
              />
              <n-input v-model:value="apiBaseUrl" style="width: 200px" />
              <n-input v-model:value="apiPath" placeholder="/path" />
              <n-button type="primary" @click="sendRequest">
                <template #icon>
                  <Send class="icon-sm" />
                </template>
                发送
              </n-button>
            </n-input-group>

            <div v-if="['POST', 'PUT', 'PATCH'].includes(selectedMethod)">
              <div class="form-section-title">请求体 (JSON)</div>
              <n-input
                v-model:value="requestBody"
                type="textarea"
                :rows="6"
                placeholder="输入 JSON 请求体..."
              />
            </div>
          </n-space>
        </n-card>

        <n-card title="响应结果">
          <template #header-extra>
            <n-space>
              <n-tag v-if="responseStatus" :type="responseStatus === 200 ? 'success' : 'error'">
                <template #icon>
                  <component :is="responseStatus === 200 ? CheckCircle : XCircle" class="icon-sm" />
                </template>
                {{ responseStatus }}
              </n-tag>
              <n-tag v-if="responseTime > 0">
                <template #icon>
                  <Clock class="icon-sm" />
                </template>
                {{ responseTime }}ms
              </n-tag>
              <n-button size="small" text @click="clearResponse">
                <template #icon>
                  <Trash2 class="icon-sm" />
                </template>
                清空
              </n-button>
            </n-space>
          </template>
          <n-input
            v-model:value="responseData"
            type="textarea"
            :rows="12"
            readonly
            placeholder="响应结果将显示在这里..."
            style="font-family: monospace; font-size: 13px"
          />
        </n-card>
      </div>
    </div>
  </div>
</template>
