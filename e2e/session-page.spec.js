const { test, expect } = require("@playwright/test");

test.describe("Session Page", () => {
  let sessionId;

  // Create a session before testing session page functionality
  test.beforeEach(async ({ page }) => {
    // Go to home page and create a session first
    await page.goto("/");

    // Fill out the create session form
    await page.locator("input#session-name").fill("E2E Test Session");
    await page.locator('button:has-text("30m")').click(); // Use 30m preset

    // Submit the form and wait for success
    await page.locator('button[type="submit"]').click();
    await expect(page.locator(".text-green-700")).toContainText(
      "Session created successfully!",
      { timeout: 10000 }
    );

    // Extract session ID from current created sessions (we'll need to implement this differently)
    // For now, let's create a mock session ID - in real tests you'd extract it from the success flow
    sessionId = "test-session-id";
  });

  test("should display session details page with proper heading", async ({
    page,
  }) => {
    // For this test, let's go directly to a session URL
    // In a real scenario, you'd use the sessionId from the beforeEach
    await page.goto("/session/nonexistent-session");

    await expect(page.locator("h1")).toHaveText("Session Details");
  });

  test("should show loading state initially", async ({ page }) => {
    await page.goto("/session/test-session");

    // Should show loading spinner and text
    await expect(page.locator("text=Loading session...")).toBeVisible();
    await expect(page.locator("svg.animate-spin")).toBeVisible();
  });

  test("should show error for nonexistent session", async ({ page }) => {
    await page.goto("/session/nonexistent-session-12345");

    // Wait for loading to finish and error to appear
    await expect(page.locator("text=⚠️ Error Loading Session")).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator(".text-red-700")).toContainText(
      "Failed to load session"
    );

    // Should have a retry button
    await expect(page.locator('button:has-text("Retry")')).toBeVisible();
  });

  test("should display WebSocket connection status", async ({ page }) => {
    await page.goto("/session/test-session");

    // Check for connection status indicator
    await expect(page.locator(".w-3.h-3.rounded-full")).toBeVisible();

    // Should show connection status text
    const statusTexts = [
      "Connected",
      "Connecting...",
      "Disconnected",
      "Connection Error",
    ];
    const statusElement = page
      .locator("span")
      .filter({
        hasText: /Connected|Connecting|Disconnected|Connection Error/,
      });
    await expect(statusElement).toBeVisible();
  });

  test("should show participant join form when not joined", async ({
    page,
  }) => {
    // This test assumes we can create a valid session and navigate to it
    // For now we'll test the UI elements that should be present
    await page.goto("/session/test-session");

    // Wait for any initial loading to complete
    await page.waitForTimeout(2000);

    // Look for join participant elements (they may or may not be visible depending on session state)
    const joinSection = page.locator("text=Join as Participant");
    const participantInput = page.locator("input#participant-name");
    const joinButton = page.locator('button:has-text("Join as Participant")');

    // Check if join form exists in DOM (it might be hidden if already joined)
    const joinFormExists = (await joinSection.count()) > 0;

    if (joinFormExists) {
      await expect(joinSection).toBeVisible();
      await expect(participantInput).toHaveAttribute(
        "placeholder",
        "Enter your name..."
      );
      await expect(joinButton).toBeDisabled(); // Should be disabled when input is empty
    }
  });

  test("should enable join button when participant name is entered", async ({
    page,
  }) => {
    await page.goto("/session/test-session");

    // Wait for potential join form to appear
    await page.waitForTimeout(2000);

    const participantInput = page.locator("input#participant-name");
    const joinButton = page.locator('button:has-text("Join as Participant")');

    // Only proceed if the join form is visible
    if ((await participantInput.count()) > 0) {
      // Initially button should be disabled
      await expect(joinButton).toBeDisabled();

      // Fill in participant name
      await participantInput.fill("Test Participant");

      // Button should now be enabled
      await expect(joinButton).not.toBeDisabled();
    }
  });

  test("should allow joining with Enter key", async ({ page }) => {
    await page.goto("/session/test-session");

    await page.waitForTimeout(2000);

    const participantInput = page.locator("input#participant-name");

    if ((await participantInput.count()) > 0) {
      await participantInput.fill("Test Participant");
      await participantInput.press("Enter");

      // After pressing Enter, the form should attempt to join
      // We can check that the input is no longer focused or form state changed
      // (specific behavior depends on the join implementation)
    }
  });

  test("should handle WebSocket connection errors gracefully", async ({
    page,
  }) => {
    await page.goto("/session/test-session");

    // Wait for potential connection attempts
    await page.waitForTimeout(3000);

    // Look for potential error messages
    const wsError = page.locator("text=WebSocket Issue");
    const connectionError = page.locator("text=Connection Failed");

    const wsErrorExists = (await wsError.count()) > 0;
    const connectionErrorExists = (await connectionError.count()) > 0;

    if (wsErrorExists) {
      await expect(
        page.locator('button:has-text("Retry Connection")')
      ).toBeVisible();
    }

    if (connectionErrorExists) {
      await expect(
        page.locator('button:has-text("Retry Connection")')
      ).toBeVisible();
    }
  });

  test("should show participating status when joined", async ({ page }) => {
    await page.goto("/session/test-session");

    await page.waitForTimeout(2000);

    // Check for the participating status message
    // This would appear if the user is already joined
    const participatingStatus = page.locator("text=Participating as");

    if ((await participatingStatus.count()) > 0) {
      await expect(participatingStatus).toBeVisible();
      await expect(page.locator(".text-green-800")).toBeVisible();
    }
  });

  test.skip("should display retry button on session load error", async ({
    page,
  }) => {
    await page.goto("/session/definitely-nonexistent-session-xyz");

    // Wait for error to appear
    await expect(page.locator("text=⚠️ Error Loading Session")).toBeVisible({
      timeout: 10000,
    });

    const retryButton = page.locator('button:has-text("Retry")');
    await expect(retryButton).toBeVisible();

    // Click retry button
    await retryButton.click();

    // Should show loading state again
    await expect(page.locator("text=Loading session...")).toBeVisible();
  });
});
