/**
 * useTheme composable unit tests
 */
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useTheme } from './useTheme'
import { useThemeStore } from '@/stores/theme'

// Mock localStorage
const localStorageMock = {
  store: {} as Record<string, string>,
  getItem: vi.fn((key: string) => localStorageMock.store[key] || null),
  setItem: vi.fn((key: string, value: string) => {
    localStorageMock.store[key] = value
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageMock.store[key]
  }),
  clear: vi.fn(() => {
    localStorageMock.store = {}
  }),
}

// Mock matchMedia
const matchMediaMock = vi.fn().mockImplementation((query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
}))

vi.stubGlobal('localStorage', localStorageMock)
vi.stubGlobal('matchMedia', matchMediaMock)

describe('useTheme', () => {
  beforeEach(() => {
    // Create a fresh pinia instance for each test
    setActivePinia(createPinia())
    // Reset localStorage mock
    localStorageMock.store = {}
    localStorageMock.getItem.mockClear()
    localStorageMock.setItem.mockClear()
    // Reset matchMedia
    matchMediaMock.mockClear()
    // Reset document class
    document.documentElement.classList.remove('dark')
  })

  it('should return mode, isDark, and setTheme', () => {
    const { mode, isDark, setTheme } = useTheme()

    expect(mode).toBeDefined()
    expect(isDark).toBeDefined()
    expect(setTheme).toBeDefined()
    expect(typeof setTheme).toBe('function')
  })

  it('should default to system mode when no theme in localStorage', () => {
    const { mode } = useTheme()
    expect(mode.value).toBe('system')
  })

  it('should read theme from localStorage', () => {
    localStorageMock.store['theme'] = 'dark'
    // Need to create a new store to read from localStorage
    setActivePinia(createPinia())

    const store = useThemeStore()
    expect(store.mode).toBe('dark')
  })

  it('should set light theme correctly', () => {
    const { setTheme, isDark, mode } = useTheme()

    setTheme('light')

    expect(mode.value).toBe('light')
    expect(isDark.value).toBe(false)
    expect(localStorageMock.setItem).toHaveBeenCalledWith('theme', 'light')
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('should set dark theme correctly', () => {
    const { setTheme, isDark, mode } = useTheme()

    setTheme('dark')

    expect(mode.value).toBe('dark')
    expect(isDark.value).toBe(true)
    expect(localStorageMock.setItem).toHaveBeenCalledWith('theme', 'dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('should handle system theme with light preference', () => {
    matchMediaMock.mockImplementation(() => ({
      matches: false,
      addEventListener: vi.fn(),
    }))
    setActivePinia(createPinia())

    const { setTheme, isDark } = useTheme()
    setTheme('system')

    expect(isDark.value).toBe(false)
  })

  it('should handle system theme with dark preference', () => {
    matchMediaMock.mockImplementation(() => ({
      matches: true,
      addEventListener: vi.fn(),
    }))
    setActivePinia(createPinia())

    const { isDark } = useTheme()
    // System mode + dark preference = dark
    expect(isDark.value).toBe(true)
  })

  it('should share state between multiple useTheme calls', () => {
    const theme1 = useTheme()
    const theme2 = useTheme()

    theme1.setTheme('dark')

    expect(theme2.mode.value).toBe('dark')
    expect(theme2.isDark.value).toBe(true)
  })
})
