const { test, expect } = require("@playwright/test");

test.describe("Create Session Functionality", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("should enable submit button when form is filled correctly", async ({
    page,
  }) => {
    const sessionNameInput = page.locator("input#session-name");
    const submitButton = page.locator('button[type="submit"]');

    // Initially disabled
    await expect(submitButton).toBeDisabled();

    // Fill in session name
    await sessionNameInput.fill("Test Session");

    // Default duration should be 30 minutes (form should be valid)
    await expect(submitButton).not.toBeDisabled();
  });

  test("should update duration when preset buttons are clicked", async ({
    page,
  }) => {
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    const minutesInput = page.locator('input[aria-label="Duration Minutes"]');

    // Click 15m preset
    await page.locator('button:has-text("15m")').click();
    await expect(hoursInput).toHaveValue("0");
    await expect(minutesInput).toHaveValue("15");

    // Click 1h preset
    await page.locator('button:has-text("1h")').click();
    await expect(hoursInput).toHaveValue("1");
    await expect(minutesInput).toHaveValue("0");

    // Click 45m preset
    await page.locator('button:has-text("45m")').click();
    await expect(hoursInput).toHaveValue("0");
    await expect(minutesInput).toHaveValue("45");
  });

  test("should allow manual duration input", async ({ page }) => {
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    const minutesInput = page.locator('input[aria-label="Duration Minutes"]');

    // Set custom duration
    await hoursInput.fill("2");
    await minutesInput.fill("15");

    await expect(hoursInput).toHaveValue("2");
    await expect(minutesInput).toHaveValue("15");
  });

  test("should keep submit button disabled for empty session name", async ({
    page,
  }) => {
    const submitButton = page.locator('button[type="submit"]');

    // Button should remain disabled with empty session name
    await expect(submitButton).toBeDisabled();
  });

  test("should keep submit button disabled for zero duration", async ({
    page,
  }) => {
    const sessionNameInput = page.locator("input#session-name");
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    const minutesInput = page.locator('input[aria-label="Duration Minutes"]');
    const submitButton = page.locator('button[type="submit"]');

    // Fill session name and set duration to 0
    await sessionNameInput.fill("Test Session");
    await hoursInput.fill("0");
    await minutesInput.fill("0");

    // Lose focus to trigger validation
    await minutesInput.blur();
    await hoursInput.blur();

    // Button should remain disabled with zero duration
    await expect(submitButton).toBeDisabled();
  });

  test.skip("should keep submit button disabled for duration over 24 hours", async ({
    page,
  }) => {
    const sessionNameInput = page.locator("input#session-name");
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    const submitButton = page.locator('button[type="submit"]');

    // Fill session name and set duration over 24 hours
    await sessionNameInput.fill("Test Session");
    await hoursInput.fill("25");

    // Button should remain disabled with duration over 24 hours
    await expect(submitButton).toBeDisabled();
  });

  test("should create session successfully and show success message", async ({
    page,
  }) => {
    const sessionNameInput = page.locator("input#session-name");
    const submitButton = page.locator('button[type="submit"]');

    // Fill form
    await sessionNameInput.fill("E2E Test Session");
    await page.locator('button:has-text("30m")').click(); // Use 30m preset

    // Submit form
    await submitButton.click();

    // Should show loading state
    await expect(submitButton).toBeDisabled();

    // Should eventually show success message
    await expect(page.locator(".text-green-700")).toContainText(
      "Session created successfully!",
      { timeout: 10000 }
    );

    // Success message should disappear after a while and form should reset
    await expect(page.locator(".text-green-700")).not.toBeVisible({
      timeout: 5000,
    });
    await expect(sessionNameInput).toHaveValue("");
  });

  test("should open new session in new tab after successful creation", async ({
    context,
    page,
  }) => {
    const sessionNameInput = page.locator("input#session-name");
    const submitButton = page.locator('button[type="submit"]');

    // Fill form
    await sessionNameInput.fill("New Tab Test Session");
    await page.locator('button:has-text("15m")').click();

    // Listen for new pages (tabs)
    const pagePromise = context.waitForEvent("page");

    // Submit form
    await submitButton.click();

    // Wait for success message
    await expect(page.locator(".text-green-700")).toContainText(
      "Session created successfully!",
      { timeout: 10000 }
    );

    // Wait for new page to open
    const newPage = await pagePromise;
    await newPage.waitForLoadState();

    // Check that new page URL contains '/session/'
    expect(newPage.url()).toContain("/session/");
  });
});
