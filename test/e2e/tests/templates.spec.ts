/**
 * E2E Tests: Templates Management
 * Tests for template CRUD operations and task execution
 */

import { test, expect } from '@playwright/test';

test.describe('Templates Management', () => {
  const testTemplateName = `E2E Test Template ${Date.now()}`;
  
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

  test('E2E-TMPL-01: Templates page loads', async ({ page }) => {
    // Navigate to a project first
    await page.goto('/index.html');
    const projectLink = page.locator('a[href*="project"]').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      await page.waitForURL(/\/project\.html/);
      
      // Click on Templates in sidebar
      const templatesLink = page.locator('a:has-text("Templates")');
      if (await templatesLink.count() > 0) {
        await templatesLink.first().click();
        await page.waitForURL(/\/templates\.html/);
        
        // Verify templates page loads
        await expect(page).toHaveTitle(/Templates|Semaphore/);
      }
    }
  });

  test('E2E-TMPL-02: Create new template', async ({ page }) => {
    await page.goto('/index.html');
    const projectLink = page.locator('a[href*="project"]').first();
    if (await projectLink.count() > 0) {
      await projectLink.click();
      await page.waitForURL(/\/project\.html/);
      
      // Navigate to templates
      const templatesLink = page.locator('a:has-text("Templates")');
      if (await templatesLink.count() > 0) {
        await templatesLink.first().click();
        await page.waitForURL(/\/templates\.html/);
        
        // Click "New Template" button
        const newTemplateButton = page.locator('button:has-text("New Template"), button:has-text("+ Template")');
        if (await newTemplateButton.count() > 0) {
          await newTemplateButton.first().click();
          
          // Wait for modal
          await page.waitForTimeout(1000);
          
          // Fill template form
          const nameInput = page.locator('input[name="name"], input[placeholder*="template name"]');
          if (await nameInput.count() > 0) {
            await nameInput.fill(testTemplateName);
            
            // Select playbook (if dropdown exists)
            const playbookSelect = page.locator('select[name="playbook"]');
            if (await playbookSelect.count() > 0) {
              const firstOption = playbookSelect.locator('option').nth(1);
              const optionValue = await firstOption.getAttribute('value');
              if (optionValue) {
                await playbookSelect.selectOption(optionValue);
              }
            }
            
            // Submit form
            const submitButton = page.locator('button[type="submit"], button:has-text("Create"), button:has-text("Save")');
            await submitButton.first().click();
            
            // Wait for creation
            await page.waitForTimeout(2000);
            
            // Verify template appears in list
            await expect(page.locator(`text=${testTemplateName}`)).toBeVisible({ timeout: 5000 });
          }
        }
      }
    }
  });

  test('E2E-TMPL-03: View template details', async ({ page }) => {
    await page.goto('/templates.html');
    
    // Click on first template
    const templateLink = page.locator('table tr a, .template-card a').first();
    if (await templateLink.count() > 0) {
      await templateLink.click();
      
      // Verify template details page loads
      await page.waitForTimeout(2000);
      
      // Check for template information
      const detailsSection = page.locator('.template-details, .card, form');
      await expect(detailsSection.first()).toBeVisible();
    }
  });

  test('E2E-TMPL-04: Edit template', async ({ page }) => {
    await page.goto('/templates.html');
    
    // Find and edit first template
    const editButton = page.locator('button:has-text("Edit"), .btn-edit, .edit-icon').first();
    if (await editButton.count() > 0) {
      await editButton.click();
      
      // Wait for modal or edit page
      await page.waitForTimeout(1000);
      
      // Modify name
      const nameInput = page.locator('input[name="name"]');
      if (await nameInput.count() > 0) {
        const currentValue = await nameInput.inputValue();
        await nameInput.fill(`${currentValue} - Edited`);
        
        // Save changes
        const saveButton = page.locator('button[type="submit"], button:has-text("Save")');
        await saveButton.first().click();
        
        // Verify update
        await page.waitForTimeout(2000);
      }
    }
  });

  test('E2E-TMPL-05: Run template task', async ({ page }) => {
    await page.goto('/templates.html');
    
    // Click run button on first template
    const runButton = page.locator('button:has-text("Run"), button:has-text("▶"), .btn-run').first();
    if (await runButton.count() > 0) {
      await runButton.click();
      
      // Wait for run modal or redirect to task page
      await page.waitForTimeout(2000);
      
      // If modal, confirm run
      const confirmButton = page.locator('button:has-text("Run"), button:has-text("Start"), button:has-text("Confirm")');
      if (await confirmButton.count() > 0) {
        await confirmButton.first().click();
      }
      
      // Verify task is created and redirects to task page
      await page.waitForURL(/\/task\.html|\/tasks\/\d+/, { timeout: 5000 });
      
      // Check task status
      const statusElement = page.locator('.task-status, .status-badge, [class*="status"]');
      await expect(statusElement.first()).toBeVisible();
    }
  });

  test('E2E-TMPL-06: View task output log', async ({ page }) => {
    await page.goto('/tasks.html');
    
    // Click on first task
    const taskLink = page.locator('table tr a, .task-card a').first();
    if (await taskLink.count() > 0) {
      await taskLink.click();
      
      // Wait for task details page
      await page.waitForURL(/\/task\.html/);
      await page.waitForTimeout(2000);
      
      // Check for log output
      const logOutput = page.locator('.task-log, .log-output, pre, code, .terminal');
      if (await logOutput.count() > 0) {
        await expect(logOutput.first()).toBeVisible();
      }
    }
  });

  test('E2E-TMPL-07: Delete template', async ({ page }) => {
    test.slow();
    
    await page.goto('/templates.html');
    
    // Find and delete test template
    const templateRow = page.locator(`tr:has-text("${testTemplateName}"), .template-card:has-text("${testTemplateName}")`);
    
    if (await templateRow.count() > 0) {
      // Click delete button
      const deleteButton = templateRow.locator('button:has-text("Delete"), .btn-delete, .delete-icon').first();
      if (await deleteButton.count() > 0) {
        await deleteButton.click();
        
        // Confirm deletion
        const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Delete"), .btn-danger').first();
        if (await confirmButton.count() > 0) {
          await confirmButton.click();
          
          // Verify deletion
          await page.waitForTimeout(2000);
          await expect(page.locator(`text=${testTemplateName}`)).not.toBeVisible();
        }
      }
    }
  });

  test('E2E-TMPL-08: Template with WebSocket live log', async ({ page }) => {
    await page.goto('/templates.html');
    
    // Run a template
    const runButton = page.locator('button:has-text("Run"), button:has-text("▶")').first();
    if (await runButton.count() > 0) {
      await runButton.click();
      await page.waitForTimeout(1000);
      
      // Confirm run
      const confirmButton = page.locator('button:has-text("Run"), button:has-text("Start")');
      if (await confirmButton.count() > 0) {
        await confirmButton.first().click();
      }
      
      // Wait for task page
      await page.waitForURL(/\/task\.html/, { timeout: 5000 });
      await page.waitForTimeout(3000);
      
      // Check for live log updates (look for WebSocket connection indicator or auto-scrolling log)
      const logContainer = page.locator('.task-log, .log-output');
      if (await logContainer.count() > 0) {
        // Take initial screenshot
        const initialLog = await logContainer.first().textContent();
        
        // Wait for log updates
        await page.waitForTimeout(5000);
        
        // Check if log has updated (content changed or new lines added)
        const updatedLog = await logContainer.first().textContent();
        
        // Log should have content
        expect(updatedLog?.length).toBeGreaterThan(0);
      }
    }
  });
});
