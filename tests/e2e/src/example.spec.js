const { test, expect } = require("@playwright/test");

test.describe("Example Playwright Tests", () => {
  test("basic page navigation and interaction", async ({ page }) => {
    // Navigate to home page
    await page.goto("/");

    // Take a screenshot
    await page.screenshot({ path: "test-screenshots/home-page.png" });

    // Check page title
    await expect(page).toHaveTitle(/Kiko/);

    // Test form interactions
    const sessionNameInput = page.locator("input#session-name");
    await sessionNameInput.fill("Integration Test Session");

    // Test keyboard interactions
    await sessionNameInput.press("Tab");

    // Test hover states
    await page.locator('button:has-text("30m")').hover();

    // Click preset button
    await page.locator('button:has-text("30m")').click();

    // Wait for potential changes
    await page.waitForTimeout(100);

    // Check that form is now valid
    const submitButton = page.locator('button[type="submit"]');
    await expect(submitButton).not.toBeDisabled();

    // Take another screenshot
    await page.screenshot({ path: "test-screenshots/filled-form.png" });
  });

  test("responsive design check", async ({ page }) => {
    await page.goto("/");

    // Test desktop viewport
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.screenshot({ path: "test-screenshots/desktop-view.png" });

    // Test tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.screenshot({ path: "test-screenshots/tablet-view.png" });

    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.screenshot({ path: "test-screenshots/mobile-view.png" });

    // Ensure form is still usable on mobile
    const sessionNameInput = page.locator("input#session-name");
    await expect(sessionNameInput).toBeVisible();

    const submitButton = page.locator('button[type="submit"]');
    await expect(submitButton).toBeVisible();
  });

  test("accessibility checks", async ({ page }) => {
    await page.goto("/");

    // Check for proper labels
    const sessionNameInput = page.locator("input#session-name");
    await expect(page.locator('label[for="session-name"]')).toBeVisible();

    // Check ARIA attributes
    const hoursInput = page.locator('input[aria-label="Duration Hours"]');
    await expect(hoursInput).toHaveAttribute("aria-label", "Duration Hours");

    const minutesInput = page.locator('input[aria-label="Duration Minutes"]');
    await expect(minutesInput).toHaveAttribute(
      "aria-label",
      "Duration Minutes"
    );

    // Test keyboard navigation
    await page.keyboard.press("Tab"); // Should focus first interactive element
    await page.keyboard.press("Tab"); // Should move to next element

    // Check focus is visible (this requires CSS focus styles)
    const focusedElement = page.locator(":focus");
    await expect(focusedElement).toBeVisible();
  });

  test("error handling and validation", async ({ page }) => {
    await page.goto("/");

    // Test form submission without required fields
    const submitButton = page.locator('button[type="submit"]');

    // Should be disabled initially
    await expect(submitButton).toBeDisabled();

    // Fill only session name
    await page.locator("input#session-name").fill("Test Session");

    // Set duration to 0
    await page.locator('input[aria-label="Duration Hours"]').fill("0");
    await page.locator('input[aria-label="Duration Minutes"]').fill("0");

    // Try to submit (should show validation error)
    await submitButton.click({ force: true });
    await page.locator('input[aria-label="Duration Minutes"]').blur();
    await page.locator('input[aria-label="Duration Hours"]').blur();

    // Expect submit button to still be disabled
    await expect(submitButton).toBeDisabled();

    // Fix the form
    await page.locator('button:has-text("15m")').click();

    // Error should clear
    await expect(page.locator(".text-red-700")).not.toBeVisible();
  });

  test("browser console errors", async ({ page }) => {
    const consoleErrors = [];

    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto("/");

    // Interact with the page
    await page.locator("input#session-name").fill("Console Test");
    await page.locator('button:has-text("30m")').click();

    // Wait a bit for any async operations
    await page.waitForTimeout(1000);

    // Check that no console errors occurred
    expect(consoleErrors).toHaveLength(0);
  });

  test("network request monitoring", async ({ page }) => {
    const requests = [];

    page.on("request", (request) => {
      requests.push({
        url: request.url(),
        method: request.method(),
      });
    });

    await page.goto("/");

    // Trigger form submission which should make API calls
    await page.locator("input#session-name").fill("Network Test");
    await page.locator('button:has-text("30m")').click();
    await page.locator('button[type="submit"]').click();

    // Wait for potential API calls
    await page.waitForTimeout(2000);

    // Check that requests were made to expected endpoints
    const apiRequests = requests.filter((req) => req.url.includes("/api/"));

    // We expect at least some API calls to be made
    // (specific assertions would depend on your API structure)
    console.log("API requests made:", apiRequests);
  });
});
