<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card, Button, Select, Loading } from '@/components/ui'
import { SettingsSaveFooter } from '@/components/settings'
import { useLocalDraft } from '@/composables/useSettingsDraft'
import { systemApi, mcpApi } from '@/lib/api'
import type { UserSystemInfo, McpConfig, PopularMcpServer } from '@/types/settings'
import { POPULAR_MCP_SERVERS } from '@/types/settings'
import { AlertTriangle, Plus, ChevronLeft, ChevronRight } from 'lucide-vue-next'

const { t } = useI18n()

// 系统信息
const systemInfo = ref<UserSystemInfo | null>(null)

// 选中的代理
const selectedAgent = ref<string>('claude-code')

// MCP 配置
const mcpConfigJson = ref<string>('{\n  "servers": {}\n}')

// 是否支持 MCP
const supportsMcp = computed(() => {
  // 假设只有 claude-code 支持 MCP
  return selectedAgent.value === 'claude-code'
})

// 热门服务器轮播索引
const carouselIndex = ref(0)
const visibleServers = computed(() => {
  const start = carouselIndex.value
  return POPULAR_MCP_SERVERS.slice(start, start + 3)
})

// 草稿管理
const { draft, hasChanges, saving, success, error, init, update, reset, save } = useLocalDraft<{
  config: string
}>()

// 加载状态
const loading = ref(true)

// 代理选项
const agentOptions = computed(() => {
  if (!systemInfo.value?.profiles?.executors) return []
  return Object.keys(systemInfo.value.profiles.executors).map(key => ({
    value: key,
    label: key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
  }))
})

// 加载系统信息和 MCP 配置
onMounted(async () => {
  await loadData()
})

// 监听代理变化
watch(selectedAgent, async () => {
  if (supportsMcp.value) {
    await loadMcpConfig()
  }
})

const loadData = async () => {
  loading.value = true
  try {
    const data = await systemApi.getInfo() as unknown as UserSystemInfo
    systemInfo.value = data
    if (data.config?.executor_profile) {
      selectedAgent.value = data.config.executor_profile
    }
    if (supportsMcp.value) {
      await loadMcpConfig()
    }
  } catch (err) {
    console.error('Failed to load data:', err)
  } finally {
    loading.value = false
  }
}

const loadMcpConfig = async () => {
  try {
    const config = await mcpApi.getConfig(selectedAgent.value)
    mcpConfigJson.value = JSON.stringify(config, null, 2)
    init({ config: mcpConfigJson.value })
  } catch (err) {
    // 如果 API 不存在或返回错误，使用默认配置
    console.error('Failed to load MCP config:', err)
    const defaultConfig = { servers: {} }
    mcpConfigJson.value = JSON.stringify(defaultConfig, null, 2)
    init({ config: mcpConfigJson.value })
  }
}

// 保存 MCP 配置
const handleSave = async () => {
  if (!draft.value) return

  await save(async (data) => {
    // 验证 JSON 格式
    let config: McpConfig
    try {
      config = JSON.parse(data.config)
    } catch {
      throw new Error('JSON 格式无效')
    }
    await mcpApi.updateConfig(selectedAgent.value, config)
  })
}

// 更新配置
const updateConfig = (value: string) => {
  mcpConfigJson.value = value
  update({ config: value })
}

// 添加热门服务器
const addPopularServer = (server: PopularMcpServer) => {
  try {
    const config = JSON.parse(mcpConfigJson.value) as McpConfig
    config.servers[server.name.toLowerCase().replace(/\s+/g, '-')] = server.config
    const newJson = JSON.stringify(config, null, 2)
    mcpConfigJson.value = newJson
    update({ config: newJson })
  } catch (err) {
    console.error('Failed to add server:', err)
  }
}

// 轮播导航
const prevCarousel = () => {
  if (carouselIndex.value > 0) {
    carouselIndex.value--
  }
}

const nextCarousel = () => {
  if (carouselIndex.value < POPULAR_MCP_SERVERS.length - 3) {
    carouselIndex.value++
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-text-primary">
        {{ t('settings.mcp.title') }}
      </h2>
    </div>

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- 代理选择 -->
      <Card>
        <div class="flex items-center gap-4">
          <label class="text-sm font-medium text-text-primary whitespace-nowrap">
            {{ t('settings.mcp.agent') }}
          </label>
          <Select
            v-model="selectedAgent"
            :options="agentOptions"
            :placeholder="t('settings.mcp.agentPlaceholder')"
            class="flex-1"
          />
        </div>
      </Card>

      <!-- MCP 不支持警告 -->
      <div
        v-if="!supportsMcp"
        class="flex items-start gap-3 p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg"
      >
        <AlertTriangle class="w-5 h-5 text-amber-500 flex-shrink-0 mt-0.5" />
        <div>
          <div class="font-medium text-amber-800 dark:text-amber-200">
            {{ t('settings.mcp.notSupported') }}
          </div>
          <div class="text-sm text-amber-700 dark:text-amber-300 mt-0.5">
            {{ t('settings.mcp.notSupportedHint') }}
          </div>
        </div>
      </div>

      <template v-if="supportsMcp">
        <!-- MCP 配置编辑器 -->
        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">
              MCP 服务器配置
            </h3>
          </template>

          <div class="space-y-4">
            <textarea
              :value="mcpConfigJson"
              @input="updateConfig(($event.target as HTMLTextAreaElement).value)"
              class="input min-h-[300px] font-mono text-sm"
              placeholder='{"servers": {}}'
            />

            <!-- 配置文件路径 -->
            <div v-if="systemInfo?.environment?.config_path">
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.mcp.configPath') }}
              </label>
              <div class="px-3 py-2 bg-bg-secondary rounded-lg font-mono text-sm text-text-normal">
                {{ systemInfo.environment.config_path }}/mcp-{{ selectedAgent }}.json
              </div>
            </div>
          </div>
        </Card>

        <!-- 热门服务器 -->
        <Card>
          <template #header>
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-text-primary">
                  {{ t('settings.mcp.popular') }}
                </h3>
                <p class="text-sm text-text-low mt-0.5">
                  {{ t('settings.mcp.popularDesc') }}
                </p>
              </div>
              <div class="flex items-center gap-1">
                <button
                  @click="prevCarousel"
                  :disabled="carouselIndex === 0"
                  class="p-1.5 rounded-lg hover:bg-bg-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <ChevronLeft class="w-4 h-4" />
                </button>
                <button
                  @click="nextCarousel"
                  :disabled="carouselIndex >= POPULAR_MCP_SERVERS.length - 3"
                  class="p-1.5 rounded-lg hover:bg-bg-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <ChevronRight class="w-4 h-4" />
                </button>
              </div>
            </div>
          </template>

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div
              v-for="server in visibleServers"
              :key="server.name"
              class="p-4 border border-border-normal rounded-lg hover:border-brand/50 transition-colors"
            >
              <div class="font-medium text-text-primary">
                {{ server.name }}
              </div>
              <div class="text-sm text-text-low mt-1">
                {{ server.description }}
              </div>
              <div class="text-xs text-text-low font-mono mt-2 truncate">
                {{ server.package }}
              </div>
              <Button
                variant="secondary"
                size="sm"
                class="mt-3 w-full"
                @click="addPopularServer(server)"
              >
                <Plus class="w-3 h-3 mr-1" />
                {{ t('settings.mcp.addServer') }}
              </Button>
            </div>
          </div>
        </Card>

        <!-- 保存底部栏 -->
        <SettingsSaveFooter
          :has-changes="hasChanges"
          :saving="saving"
          :success="success"
          :error="error"
          @save="handleSave"
          @discard="reset"
        />
      </template>
    </template>
  </div>
</template>
