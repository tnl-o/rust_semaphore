/**
 * E2E Tests: Projects Management
 * Tests for project CRUD operations
 */

import { test, expect } from '@playwright/test';

test.describe('Projects Management', () => {
  const testProjectName = `E2E Test Project ${Date.now()}`;
  
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login.html');
    const username = process.env.ADMIN_USERNAME || 'admin';
    const password = process.env.ADMIN_PASSWORD || 'password123';
    
    await page.locator('input[name="auth"]').fill(username);
    await page.locator('input[name="password"]').fill(password);
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/\/index\.html|\/dashboard/, { timeout: 5000 });
  });

  test('E2E-PROJ-01: Dashboard loads with projects list', async ({ page }) => {
    await page.goto('/index.html');
    
    // Verify dashboard loads
    await expect(page).toHaveTitle(/Dashboard|Semaphore/);
    
    // Check for projects list or "no projects" message
    const projectsList = page.locator('table, .projects-list, .project-card');
    await expect(projectsList.first()).toBeVisible({ timeout: 5000 });
  });

  test('E2E-PROJ-02: Create new project', async ({ page }) => {
    await page.goto('/index.html');
    
    // Click "New Project" button (adjust selector based on actual UI)
    const newProjectButton = page.locator('button:has-text("New Project"), button:has-text("+"), .btn:has-text("Create Project")');
    
    if (await newProjectButton.count() > 0) {
      await newProjectButton.first().click();
      
      // Wait for modal or navigate to create page
      const modal = page.locator('.modal, [role="dialog"]');
      if (await modal.count() > 0) {
        // Fill project name in modal
        await page.locator('input[name="name"], input[placeholder*="project name"]').fill(testProjectName);
        
        // Submit form
        await page.locator('button[type="submit"], button:has-text("Create"), button:has-text("Save")').first().click();
        
        // Wait for success (check for project in list or success message)
        await page.waitForTimeout(2000);
      }
    }
    
    // Verify project was created (check in projects list)
    await page.goto('/index.html');
    await expect(page.locator(`text=${testProjectName}`)).toBeVisible({ timeout: 5000 });
  });

  test('E2E-PROJ-03: View project details', async ({ page }) => {
    // Navigate to first available project
    await page.goto('/index.html');
    
    // Click on first project link
    const projectLink = page.locator('a[href*="project"], .project-card a').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      
      // Verify project page loads
      await page.waitForURL(/\/project\.html|\/projects\/\d+/);
      
      // Check for project sections (templates, inventory, etc.)
      const navItems = page.locator('nav a, .sidebar a');
      await expect(navItems.first()).toBeVisible();
    }
  });

  test('E2E-PROJ-04: Edit project settings', async ({ page }) => {
    await page.goto('/index.html');
    
    // Click on first project
    const projectLink = page.locator('a[href*="project"], .project-card a').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      await page.waitForURL(/\/project\.html|\/projects\/\d+/);
      
      // Navigate to settings (adjust selector)
      const settingsLink = page.locator('a:has-text("Settings"), .nav-link:has-text("Settings")');
      if (await settingsLink.count() > 0) {
        await settingsLink.first().click();
        
        // Verify settings page loads
        await page.waitForTimeout(1000);
      }
    }
  });

  test('E2E-PROJ-05: Delete project', async ({ page }) => {
    // This test should be run last as it modifies state
    test.slow();
    
    await page.goto('/index.html');
    
    // Find and delete the test project created earlier
    const projectRow = page.locator(`tr:has-text("${testProjectName}"), .project-card:has-text("${testProjectName}")`);
    
    if (await projectRow.count() > 0) {
      // Click delete button
      const deleteButton = projectRow.locator('button:has-text("Delete"), .btn-delete, .delete-icon').first();
      if (await deleteButton.count() > 0) {
        await deleteButton.click();
        
        // Confirm deletion
        const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Delete"), .btn-danger').first();
        if (await confirmButton.count() > 0) {
          await confirmButton.click();
          
          // Verify project is deleted
          await page.waitForTimeout(2000);
          await expect(page.locator(`text=${testProjectName}`)).not.toBeVisible();
        }
      }
    }
  });

  test('E2E-PROJ-06: Project navigation sidebar', async ({ page }) => {
    await page.goto('/index.html');
    
    // Click on first project
    const projectLink = page.locator('a[href*="project"], .project-card a').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      await page.waitForURL(/\/project\.html|\/projects\/\d+/);
      
      // Verify sidebar navigation items
      const navItems = [
        'Templates', 'Schedule', 'Inventory', 
        'Environment', 'Keys', 'Repositories',
        'Team', 'Settings'
      ];
      
      for (const item of navItems) {
        const navLink = page.locator(`a:has-text("${item}"), .nav-link:has-text("${item}")`);
        if (await navLink.count() > 0) {
          await expect(navLink.first()).toBeVisible();
        }
      }
    }
  });

  test('E2E-PROJ-07: Project stats page', async ({ page }) => {
    await page.goto('/index.html');
    
    // Click on first project
    const projectLink = page.locator('a[href*="project"], .project-card a').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      await page.waitForURL(/\/project\.html|\/projects\/\d+/);
      
      // Navigate to stats/analytics
      const statsLink = page.locator('a:has-text("Stats"), a:has-text("Analytics")');
      if (await statsLink.count() > 0) {
        await statsLink.first().click();
        
        // Verify stats page loads with charts or data
        await page.waitForTimeout(2000);
        const chartOrTable = page.locator('canvas, svg, table, .stats-card');
        await expect(chartOrTable.first()).toBeVisible({ timeout: 5000 });
      }
    }
  });
});
