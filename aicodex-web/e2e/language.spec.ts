import { test, expect } from '@playwright/test'

test.describe('Language Switcher', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/')
    await page.evaluate(() => localStorage.clear())
    await page.reload()
  })

  test('should have language switcher', async ({ page }) => {
    await page.goto('/')

    // Language switcher should be visible
    const langSwitcher = page.locator('[data-testid="language-switcher"]')
    await expect(langSwitcher).toBeVisible()
  })

  test('should switch to English', async ({ page }) => {
    await page.goto('/')

    // Open language dropdown
    await page.click('[data-testid="language-switcher"]')

    // Select English
    await page.click('text=English')

    // Navigation should show English text
    await expect(page.locator('text=Home')).toBeVisible()
  })

  test('should switch to Chinese', async ({ page }) => {
    await page.goto('/')

    // First switch to English
    await page.click('[data-testid="language-switcher"]')
    await page.click('text=English')

    // Then switch back to Chinese
    await page.click('[data-testid="language-switcher"]')
    await page.click('text=中文')

    // Navigation should show Chinese text
    await expect(page.locator('text=首页')).toBeVisible()
  })

  test('should persist language preference', async ({ page }) => {
    await page.goto('/')

    // Switch to English
    await page.click('[data-testid="language-switcher"]')
    await page.click('text=English')

    // Reload page
    await page.reload()

    // Language should still be English
    await expect(page.locator('text=Home')).toBeVisible()
  })

  test('should translate all navigation items', async ({ page }) => {
    await page.goto('/')

    // Switch to English
    await page.click('[data-testid="language-switcher"]')
    await page.click('text=English')

    // Check translated navigation items
    await expect(page.locator('nav >> text=Home')).toBeVisible()
    await expect(page.locator('nav >> text=Code Reviews')).toBeVisible()
    await expect(page.locator('nav >> text=Agent Tasks')).toBeVisible()
  })
})
