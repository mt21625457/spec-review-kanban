import { test, expect } from '@playwright/test'

test.describe('Theme Toggle', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/')
    await page.evaluate(() => localStorage.clear())
    await page.reload()
  })

  test('should have theme toggle button', async ({ page }) => {
    await page.goto('/')

    // Theme toggle should be visible
    const themeToggle = page.locator('[data-testid="theme-toggle"]')
    await expect(themeToggle).toBeVisible()
  })

  test('should toggle to dark mode', async ({ page }) => {
    await page.goto('/')

    // Click theme toggle to open dropdown
    await page.click('[data-testid="theme-toggle"]')

    // Click dark mode option
    await page.click('text=深色')

    // Check that dark class is added to html
    const isDark = await page.evaluate(() =>
      document.documentElement.classList.contains('dark')
    )
    expect(isDark).toBe(true)
  })

  test('should toggle to light mode', async ({ page }) => {
    await page.goto('/')

    // First set to dark mode
    await page.click('[data-testid="theme-toggle"]')
    await page.click('text=深色')

    // Then set to light mode
    await page.click('[data-testid="theme-toggle"]')
    await page.click('text=浅色')

    // Check that dark class is removed
    const isDark = await page.evaluate(() =>
      document.documentElement.classList.contains('dark')
    )
    expect(isDark).toBe(false)
  })

  test('should persist theme preference', async ({ page }) => {
    await page.goto('/')

    // Set dark mode
    await page.click('[data-testid="theme-toggle"]')
    await page.click('text=深色')

    // Reload page
    await page.reload()

    // Theme should still be dark
    const isDark = await page.evaluate(() =>
      document.documentElement.classList.contains('dark')
    )
    expect(isDark).toBe(true)
  })

  test('should have system theme option', async ({ page }) => {
    await page.goto('/')

    // Open theme dropdown
    await page.click('[data-testid="theme-toggle"]')

    // System option should be visible
    await expect(page.locator('text=系统')).toBeVisible()
  })
})
