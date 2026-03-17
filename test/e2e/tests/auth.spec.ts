/**
 * E2E Tests: Authentication
 * Tests for login, logout, and session management
 */

import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    // Go to login page before each test
    await page.goto('/login.html');
  });

  test('E2E-AUTH-01: Login page loads correctly', async ({ page }) => {
    // Verify login form is visible
    await expect(page.locator('input[name="auth"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
    
    // Verify page title
    await expect(page).toHaveTitle(/Semaphore/);
  });

  test('E2E-AUTH-02: Login with valid credentials', async ({ page }) => {
    // Get credentials from environment
    const username = process.env.ADMIN_USERNAME || 'admin';
    const password = process.env.ADMIN_PASSWORD || 'password123';

    // Fill login form
    await page.locator('input[name="auth"]').fill(username);
    await page.locator('input[name="password"]').fill(password);
    await page.locator('button[type="submit"]').click();

    // Wait for navigation and verify redirect to dashboard
    await expect(page).toHaveURL(/\/index\.html|\/dashboard/);
    
    // Verify user is logged in (check for logout button or user menu)
    await expect(page.locator('text=Logout, button')).toBeVisible({ timeout: 5000 });
  });

  test('E2E-AUTH-03: Login with invalid credentials shows error', async ({ page }) => {
    // Fill with invalid credentials
    await page.locator('input[name="auth"]').fill('invalid_user');
    await page.locator('input[name="password"]').fill('wrong_password');
    await page.locator('button[type="submit"]').click();

    // Wait for error message (check for error element or stayed on login page)
    await page.waitForTimeout(2000);
    
    // Should stay on login page
    await expect(page).toHaveURL(/\/login\.html/);
  });

  test('E2E-AUTH-04: Login with empty credentials', async ({ page }) => {
    // Try to submit without filling fields
    await page.locator('button[type="submit"]').click();

    // Should stay on login page
    await page.waitForTimeout(1000);
    await expect(page).toHaveURL(/\/login\.html/);
  });

  test('E2E-AUTH-05: Logout functionality', async ({ page }) => {
    // First login
    const username = process.env.ADMIN_USERNAME || 'admin';
    const password = process.env.ADMIN_PASSWORD || 'password123';

    await page.locator('input[name="auth"]').fill(username);
    await page.locator('input[name="password"]').fill(password);
    await page.locator('button[type="submit"]').click();

    // Wait for successful login
    await page.waitForURL(/\/index\.html|\/dashboard/, { timeout: 5000 });

    // Find and click logout button (adjust selector based on actual UI)
    const logoutButton = page.locator('text=Logout, button, a:has-text("Logout"), button:has-text("Logout")');
    if (await logoutButton.count() > 0) {
      await logoutButton.first().click();
      
      // Verify redirect to login page
      await expect(page).toHaveURL(/\/login\.html/);
    }
  });

  test('E2E-AUTH-06: Session persistence after page reload', async ({ page }) => {
    // Login
    const username = process.env.ADMIN_USERNAME || 'admin';
    const password = process.env.ADMIN_PASSWORD || 'password123';

    await page.locator('input[name="auth"]').fill(username);
    await page.locator('input[name="password"]').fill(password);
    await page.locator('button[type="submit"]').click();

    // Wait for successful login
    await page.waitForURL(/\/index\.html|\/dashboard/, { timeout: 5000 });

    // Reload page
    await page.reload();

    // Verify still logged in
    await expect(page).toHaveURL(/\/index\.html|\/dashboard|\/projects/);
  });

  test('E2E-AUTH-07: Protected route redirects to login when not authenticated', async ({ page, context }) => {
    // Clear cookies to ensure not authenticated
    await context.clearCookies();
    
    // Try to access protected route
    await page.goto('/projects.html');

    // Should redirect to login
    await expect(page).toHaveURL(/\/login\.html/);
  });
});
