import { test, expect } from '@playwright/test'

test.describe('Navigation', () => {
  test('should display top navigation bar', async ({ page }) => {
    await page.goto('/')

    // Check navbar is visible
    const navbar = page.locator('nav')
    await expect(navbar).toBeVisible()
  })

  test('should navigate to dashboard', async ({ page }) => {
    await page.goto('/')

    // Click dashboard link
    await page.click('text=首页')

    // Verify we're on the dashboard
    await expect(page).toHaveURL('/')
  })

  test('should navigate to reviews page', async ({ page }) => {
    await page.goto('/')

    // Click reviews link
    await page.click('text=代码审核')

    // Verify navigation
    await expect(page).toHaveURL('/reviews')
  })

  test('should navigate to tasks page', async ({ page }) => {
    await page.goto('/')

    // Click tasks link
    await page.click('text=智能体任务')

    // Verify navigation
    await expect(page).toHaveURL('/tasks')
  })

  test('should highlight active navigation item', async ({ page }) => {
    await page.goto('/tasks')

    // The tasks link should be highlighted
    const tasksLink = page.locator('nav a[href="/tasks"]')
    await expect(tasksLink).toHaveClass(/text-brand|bg-brand/)
  })

  test('should redirect old routes', async ({ page }) => {
    // Test /dashboard redirect
    await page.goto('/dashboard')
    await expect(page).toHaveURL('/')
  })

  test('should show mobile navigation on small screens', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')

    // Mobile menu button should be visible
    const menuButton = page.locator('button[aria-label="打开菜单"]')
    await expect(menuButton).toBeVisible()
  })
})
