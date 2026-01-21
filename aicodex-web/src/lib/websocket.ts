/**
 * WebSocket connection management for real-time data sync
 */

import { ref, computed, watch, onUnmounted, shallowRef, type Ref, type ShallowRef } from 'vue'
import { useWebSocket, type UseWebSocketOptions } from '@vueuse/core'
import { applyPatch, type Operation } from 'fast-json-patch'
import type { Task, TasksStreamMessage } from '@/types'

// WebSocket connection states (subset of what vueuse provides)
export type ConnectionStatus = 'CONNECTING' | 'OPEN' | 'CLOSED'

/**
 * Get WebSocket URL with correct protocol
 */
export function getWebSocketUrl(path: string): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const host = window.location.host
  return `${protocol}//${host}${path}`
}

/**
 * Default WebSocket options
 */
const defaultOptions: UseWebSocketOptions = {
  autoReconnect: {
    retries: 5,
    delay: 1000,
    onFailed() {
      console.error('[WebSocket] Failed to reconnect after max retries')
    },
  },
  heartbeat: {
    message: 'ping',
    interval: 30000,
    pongTimeout: 5000,
  },
}

/**
 * Map vueuse WebSocket status to our ConnectionStatus
 */
function mapStatus(status: string): ConnectionStatus {
  if (status === 'CONNECTING') return 'CONNECTING'
  if (status === 'OPEN') return 'OPEN'
  return 'CLOSED'
}

/**
 * Use WebSocket with auto-reconnect for tasks stream
 */
export function useTasksStream(projectId: Ref<string | null>) {
  const tasks = ref<Record<string, Task>>({})
  const lastUpdate = ref<Date | null>(null)
  const error = ref<Error | null>(null)

  const wsUrl = computed(() => {
    if (!projectId.value) return ''
    return getWebSocketUrl(`/api/proxy/tasks/stream/ws?project_id=${projectId.value}`)
  })

  const { status, data, send, open, close } = useWebSocket(wsUrl, {
    ...defaultOptions,
    immediate: false,
    onConnected() {
      console.log('[WebSocket] Tasks stream connected')
      error.value = null
    },
    onDisconnected(_ws, event) {
      console.log('[WebSocket] Tasks stream disconnected', event.reason)
    },
    onError(_ws, event) {
      console.error('[WebSocket] Tasks stream error', event)
      error.value = new Error('WebSocket connection error')
    },
  })

  // Parse incoming messages
  watch(data, (newData) => {
    if (!newData) return

    try {
      const message: TasksStreamMessage = JSON.parse(newData)

      if ('Snapshot' in message) {
        // Full snapshot - replace all tasks
        tasks.value = message.Snapshot.tasks
        lastUpdate.value = new Date(message.Snapshot.timestamp)
      } else if ('JsonPatch' in message) {
        // Incremental update - apply JSON patch
        const result = applyPatch(
          tasks.value,
          message.JsonPatch as Operation[],
          true, // validate
          false // mutate in place
        )
        tasks.value = result.newDocument as Record<string, Task>
        lastUpdate.value = new Date()
      }
    } catch (e) {
      console.error('[WebSocket] Failed to parse message', e)
      error.value = e instanceof Error ? e : new Error('Failed to parse message')
    }
  })

  // Auto-connect when projectId changes
  watch(
    projectId,
    (newId) => {
      if (newId) {
        open()
      } else {
        close()
        tasks.value = {}
      }
    },
    { immediate: true }
  )

  // Cleanup on unmount
  onUnmounted(() => {
    close()
  })

  const connectionStatus = computed<ConnectionStatus>(() => mapStatus(status.value))

  return {
    tasks,
    status: connectionStatus,
    lastUpdate,
    error,
    send,
    reconnect: open,
    disconnect: close,
  }
}

/**
 * Use WebSocket for execution logs stream
 */
export function useExecutionLogsStream(attemptId: Ref<string | null>) {
  const logs = ref<string[]>([])
  const error = ref<Error | null>(null)

  const wsUrl = computed(() => {
    if (!attemptId.value) return ''
    return getWebSocketUrl(`/api/proxy/task-attempts/${attemptId.value}/logs/ws`)
  })

  const { status, data, open, close } = useWebSocket(wsUrl, {
    ...defaultOptions,
    immediate: false,
    onConnected() {
      console.log('[WebSocket] Logs stream connected')
      error.value = null
      logs.value = []
    },
    onError(_ws, event) {
      console.error('[WebSocket] Logs stream error', event)
      error.value = new Error('WebSocket connection error')
    },
  })

  // Append incoming log lines
  watch(data, (newData) => {
    if (!newData) return
    logs.value.push(newData)
  })

  // Auto-connect when attemptId changes
  watch(
    attemptId,
    (newId) => {
      if (newId) {
        logs.value = []
        open()
      } else {
        close()
      }
    },
    { immediate: true }
  )

  onUnmounted(() => {
    close()
  })

  const connectionStatus = computed<ConnectionStatus>(() => mapStatus(status.value))

  return {
    logs,
    status: connectionStatus,
    error,
    reconnect: open,
    disconnect: close,
    clearLogs: () => {
      logs.value = []
    },
  }
}

/**
 * Generic WebSocket hook
 */
export function useGenericWebSocket<T>(
  urlOrRef: string | Ref<string>,
  options?: Partial<UseWebSocketOptions>
): {
  messages: ShallowRef<T[]>
  lastMessage: ShallowRef<T | null>
  status: Ref<ConnectionStatus>
  error: Ref<Error | null>
  send: (data: string | ArrayBuffer | Blob, useBuffer?: boolean) => boolean
  open: () => void
  close: () => void
  clear: () => void
} {
  const messages = shallowRef<T[]>([])
  const lastMessage = shallowRef<T | null>(null)
  const error = ref<Error | null>(null)

  const wsUrl = computed(() => {
    const url = typeof urlOrRef === 'string' ? urlOrRef : urlOrRef.value
    return url ? getWebSocketUrl(url) : ''
  })

  const { status, data, send, open, close } = useWebSocket(wsUrl, {
    ...defaultOptions,
    ...options,
    onError(_ws, event) {
      console.error('[WebSocket] Error', event)
      error.value = new Error('WebSocket connection error')
      options?.onError?.(_ws, event)
    },
  })

  watch(data, (newData) => {
    if (!newData) return
    try {
      const parsed = JSON.parse(newData) as T
      messages.value = [...messages.value, parsed]
      lastMessage.value = parsed
    } catch {
      // If not JSON, store as-is if T is string
      if (typeof newData === 'string') {
        messages.value = [...messages.value, newData as T]
        lastMessage.value = newData as T
      }
    }
  })

  onUnmounted(() => {
    close()
  })

  const connectionStatus = computed<ConnectionStatus>(() => mapStatus(status.value))

  return {
    messages,
    lastMessage,
    status: connectionStatus,
    error,
    send,
    open,
    close,
    clear: () => {
      messages.value = []
      lastMessage.value = null
    },
  }
}
