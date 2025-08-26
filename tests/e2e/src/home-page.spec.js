const { test, expect } = require('@playwright/test');

test.describe('Home Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display the main heading', async ({ page }) => {
    await expect(page.locator('h1')).toHaveText('Kiko Pointing Poker');
  });

  test('should display create session form', async ({ page }) => {
    await expect(page.locator('h2')).toHaveText('Create New Session');
    await expect(page.locator('input#session-name')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should have session name input with proper attributes', async ({ page }) => {
    const sessionNameInput = page.locator('input#session-name');
    await expect(sessionNameInput).toHaveAttribute('placeholder', 'Sprint Planning, Story Estimation...');
    await expect(sessionNameInput).toHaveAttribute('type', 'text');
  });

  test('should display duration inputs', async ({ page }) => {
    // Check for hours input
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    await expect(hoursInput).toBeVisible();
    await expect(hoursInput).toHaveAttribute('type', 'number');
    await expect(hoursInput).toHaveAttribute('min', '0');
    await expect(hoursInput).toHaveAttribute('max', '24');

    // Check for minutes input
    const minutesInput = page.locator('input[aria-label="Duration Minutes"]');
    await expect(minutesInput).toBeVisible();
    await expect(minutesInput).toHaveAttribute('type', 'number');
    await expect(minutesInput).toHaveAttribute('min', '0');
    await expect(minutesInput).toHaveAttribute('max', '59');
  });

  test('should display duration preset buttons', async ({ page }) => {
    const presetButtons = ['10m', '15m', '30m', '45m', '1h'];
    
    for (const preset of presetButtons) {
      await expect(page.locator(`button:has-text("${preset}")`)).toBeVisible();
    }
  });

  test('should have submit button initially disabled when form is empty', async ({ page }) => {
    const submitButton = page.locator('button[type="submit"]');
    await expect(submitButton).toBeDisabled();
  });
});