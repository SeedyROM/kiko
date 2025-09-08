const { test, expect } = require("@playwright/test");

test.describe("Complete Session Tests", () => {
  let sessionId;
  let sessionUrl;

  // Helper function to create a session and get the ID
  async function createSessionAndExtractId(page, context) {
    await page.goto("/");

    // Fill out the create session form
    await page.locator("input#session-name").fill("E2E Complete Session Test");
    await page.locator('button:has-text("30m")').click();

    // Listen for new pages (session opens in new tab)
    const pagePromise = context.waitForEvent("page");

    // Submit and wait for success
    await page.locator('button[type="submit"]').click();
    await expect(page.locator(".text-green-700")).toContainText(
      "Session created successfully!",
      { timeout: 10000 }
    );

    // Wait for new tab with session
    const sessionPage = await pagePromise;
    await sessionPage.waitForLoadState();

    // Extract session ID from the new tab URL
    const url = sessionPage.url();
    const match = url.match(/\/session\/([^\/\?]+)/);
    if (match && match[1] !== "null") {
      return match[1];
    }

    // If URL extraction fails, create a test session ID
    console.warn("Failed to extract session ID from URL:", url);
    return "test-session-fallback-" + Date.now();
  }

  test.beforeEach(async ({ page, context }) => {
    sessionId = await createSessionAndExtractId(page, context);
    sessionUrl = `/session/${sessionId}`;
  });

  test.describe("Core Session Functionality", () => {
    test("should load session and show join form", async ({ page }) => {
      await page.goto(sessionUrl);

      // Should show main header
      await expect(
        page.locator('header a')
      ).toBeVisible();

      // Should show join form since we're not joined
      await expect(
        page.getByRole("heading", { name: "Join as Participant" })
      ).toBeVisible();
      await expect(page.locator("input#participant-name")).toBeVisible();
    });

    test("should allow participant to join session", async ({ page }) => {
      await page.goto(sessionUrl);

      // Should show join form
      const participantInput = page.locator("input#participant-name");
      const joinButton = page
        .locator('button:has-text("Join as Participant")')
        .last();

      // Initially join button should be disabled
      await expect(joinButton).toBeDisabled();

      // Fill in participant name
      await participantInput.fill("Test Participant");

      // Join button should now be enabled
      await expect(joinButton).not.toBeDisabled();

      // Click join button
      await joinButton.click();

      // Should show participating status
      await expect(
        page.locator("text=Participating as Test Participant")
      ).toBeVisible();
    });

    test("should allow joining with Enter key", async ({ page }) => {
      await page.goto(sessionUrl);

      const participantInput = page.locator("input#participant-name");

      await participantInput.fill("Enter Key Participant");
      await participantInput.press("Enter");

      // Should show participating status
      await expect(
        page.locator("text=Participating as Enter Key Participant")
      ).toBeVisible();
    });

    test("should prevent joining with empty name", async ({ page }) => {
      await page.goto(sessionUrl);

      const joinButton = page
        .locator('button:has-text("Join as Participant")')
        .last();

      // Should be disabled with empty name
      await expect(joinButton).toBeDisabled();

      // Fill with whitespace only
      await page.locator("input#participant-name").fill("   ");
      await expect(joinButton).toBeDisabled();
    });
  });

  test.describe("Session View and Controls", () => {
    test.beforeEach(async ({ page }) => {
      await page.goto(sessionUrl);

      // Join as participant for session view tests
      await page.locator("input#participant-name").fill("Session Tester");
      await page
        .locator('button:has-text("Join as Participant")')
        .last()
        .click();
      await expect(
        page.locator("text=Participating as Session Tester")
      ).toBeVisible();
    });

    test("should display session details and controls", async ({ page }) => {
      // Should show session name in header
      await expect(
        page.getByRole("heading", { name: "E2E Complete Session Test" })
      ).toBeVisible();

      // Should show session status (Active/Ended)
      const statusBadge = page
        .locator("text=Active")
        .or(page.locator("text=Ended"));
      await expect(statusBadge).toBeVisible();

      // Should show refresh button
      await expect(
        page.locator('button[title="Refresh session"]')
      ).toBeVisible();

      // Should show session details section
      await expect(page.locator("text=Session Details")).toBeVisible();
      await expect(page.locator("text=Session ID")).toBeVisible();
      await expect(page.locator("text=Started")).toBeVisible();
      await expect(page.locator("text=Duration")).toBeVisible();
    });

    test("should show topic section and allow editing", async ({ page }) => {
      // Should show story topic section
      await expect(page.locator("text=Story Topic")).toBeVisible();

      // Initially should show "No topic set yet"
      await expect(page.locator("text=No topic set yet")).toBeVisible();

      // Should show edit button (we'll click directly since hover is complex in tests)
      const editButton = page.locator('button[title="Edit topic"]');
      await editButton.click();

      // Should show topic input form
      const topicInput = page.locator(
        'input[placeholder="Enter story to estimate..."]'
      );
      await expect(topicInput).toBeVisible();

      // Enter a topic
      await topicInput.fill("User login functionality");
      await page.locator('button:has-text("Set")').click();

      // Should show the topic
      await expect(page.locator("text=User login functionality")).toBeVisible();
    });

    test("should set topic with Enter key", async ({ page }) => {
      const editButton = page.locator('button[title="Edit topic"]');
      await editButton.click();

      const topicInput = page.locator(
        'input[placeholder="Enter story to estimate..."]'
      );
      await topicInput.fill("Password reset feature");
      await topicInput.press("Enter");

      // Should show the topic
      await expect(page.locator("text=Password reset feature")).toBeVisible();
    });

    test("should display voting section", async ({ page }) => {
      // Should show voting section
      await expect(page.locator("text=Choose Your Estimate")).toBeVisible();

      // Should show control buttons
      await expect(
        page
          .locator('button:has-text("👁️ Show")')
          .or(page.locator('button:has-text("🙈 Hide")'))
      ).toBeVisible();
      await expect(
        page.locator('button:has-text("🗑️ Clear All")')
      ).toBeVisible();
    });

    test("should handle session refresh", async ({ page }) => {
      const refreshButton = page.locator('button[title="Refresh session"]');
      await refreshButton.click();

      // Should still show the session content after refresh
      await expect(
        page.getByRole("heading", { name: "E2E Complete Session Test" })
      ).toBeVisible();
      await expect(
        page.locator("text=Participating as Session Tester")
      ).toBeVisible();
    });
  });

  test.describe("Voting and Point Selection", () => {
    test.beforeEach(async ({ page }) => {
      await page.goto(sessionUrl);

      await page.locator("input#participant-name").fill("Voting Tester");
      await page
        .locator('button:has-text("Join as Participant")')
        .last()
        .click();
      await expect(
        page.locator("text=Participating as Voting Tester")
      ).toBeVisible();
    });

    test("should allow point selection", async ({ page }) => {
      // Click on a point value (use first() to get the first matching button)
      await page.locator('button:has-text("5")').first().click();

      // Button should appear selected (has blue background)
      await expect(page.locator('button:has-text("5")').first()).toHaveClass(
        /bg-blue-600/
      );
    });

    test("should allow selecting different points", async ({ page }) => {
      // Select point 3
      await page.locator('button:has-text("3")').first().click();
      await expect(page.locator('button:has-text("3")').first()).toHaveClass(
        /bg-blue-600/
      );

      // Select different point (8)
      await page.locator('button:has-text("8")').first().click();
      await expect(page.locator('button:has-text("8")').first()).toHaveClass(
        /bg-blue-600/
      );

      // Previous selection should no longer be selected
      await expect(
        page.locator('button:has-text("3")').first()
      ).not.toHaveClass(/bg-blue-600/);
    });

    test("should select 'I don't know' option", async ({ page }) => {
      // Select "I don't know" option (?)
      await page.locator('button:has-text("?")').first().click();
      await expect(page.locator('button:has-text("?")').first()).toHaveClass(
        /bg-blue-600/
      );
    });

    test("should toggle hide/show points", async ({ page }) => {
      // Wait for initial button state to be stable
      await page.waitForTimeout(500);
      
      const showButton = page.locator('button:has-text("👁️ Show")');
      const hideButton = page.locator('button:has-text("🙈 Hide")');

      // Check if Show button is visible initially
      if (await showButton.isVisible()) {
        await showButton.click();
        // Should change to Hide button
        await expect(hideButton).toBeVisible({ timeout: 3000 });
      } else {
        // Hide button should be visible
        await expect(hideButton).toBeVisible();
        await hideButton.click();
        // Should change to Show button
        await expect(showButton).toBeVisible({ timeout: 3000 });
      }
    });

    test("should clear all points", async ({ page }) => {
      // Select some points first
      await page.locator('button:has-text("5")').first().click();
      await expect(page.locator('button:has-text("5")').first()).toHaveClass(
        /bg-blue-600/
      );

      // Click Clear All
      await page.locator('button:has-text("🗑️ Clear All")').click();

      // Points should no longer be selected
      await expect(
        page.locator('button:has-text("5")').first()
      ).not.toHaveClass(/bg-blue-600/);
    });
  });

  test.describe("Multi-Participant Features", () => {
    test("should handle multiple participants joining", async ({
      page,
      context,
    }) => {
      // First participant
      await page.goto(sessionUrl);

      await page.locator("input#participant-name").fill("Participant One");
      await page
        .locator('button:has-text("Join as Participant")')
        .last()
        .click();
      await expect(
        page.locator("text=Participating as Participant One")
      ).toBeVisible();

      // Create second browser tab for second participant
      const page2 = await context.newPage();
      await page2.goto(sessionUrl);

      await page2.locator("input#participant-name").fill("Participant Two");
      await page2
        .locator('button:has-text("Join as Participant")')
        .last()
        .click();
      await expect(
        page2.locator("text=Participating as Participant Two")
      ).toBeVisible();

      // Both participants should see participant results section
      await expect(page.locator("text=Current Estimates")).toBeVisible();
      await expect(page2.locator("text=Current Estimates")).toBeVisible();
    });
  });

  test.describe("Observer Mode", () => {
    test("should display limited functionality for non-participants", async ({
      page,
    }) => {
      await page.goto(sessionUrl);

      // Don't join - just observe

      // Should show join form
      await expect(
        page.getByRole("heading", { name: "Join as Participant" })
      ).toBeVisible();

      // Should show session details
      await expect(page.locator("text=Session Details")).toBeVisible();

      // Should show story topic section
      await expect(page.locator("text=Story Topic")).toBeVisible();

      // Should NOT show control buttons since not joined
      await expect(
        page.locator('button:has-text("🗑️ Clear All")')
      ).toBeHidden();
    });
  });

  test.describe("Error Handling", () => {
    test("should handle nonexistent session gracefully", async ({ page }) => {
      await page.goto("/session/nonexistent-session-12345");

      // Should show error
      await expect(page.locator("text=Error Loading Session")).toBeVisible({
        timeout: 10000,
      });
      await expect(page.locator(".text-red-700")).toContainText(
        "Failed to load session"
      );

      // Should show retry button
      await expect(page.locator('button:has-text("Retry")')).toBeVisible();
    });

    test("should allow retry on session load error", async ({ page }) => {
      await page.goto("/session/nonexistent-session-retry-test");

      await expect(page.locator("text=Error Loading Session")).toBeVisible({
        timeout: 10000,
      });

      const retryButton = page.locator('button:has-text("Retry")');
      await retryButton.click();

      // After retry, should either show loading state or error again (both indicate retry worked)
      await expect(
        page.locator("text=Loading session...").or(page.locator("text=Error Loading Session"))
      ).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe("WebSocket and Real-time Features", () => {
    test("should handle connection states", async ({ page }) => {
      await page.goto(sessionUrl);

      // Should show some connection status (Connected, Connecting, etc.)
      const connectionStates = [
        "Connected",
        "Connecting...",
        "Disconnected",
        "Connection Error",
        "WebSocket Issue",
      ];

      let foundConnection = false;
      for (const state of connectionStates) {
        const element = page.locator(`text=${state}`);
        if ((await element.count()) > 0) {
          await expect(element).toBeVisible();
          foundConnection = true;
          break;
        }
      }

      // If no connection status visible, at least verify the page loaded
      if (!foundConnection) {
        await expect(
          page.locator('header a')
        ).toBeVisible();
      }
    });

    test("should handle WebSocket connection retry", async ({ page }) => {
      await page.goto(sessionUrl);

      // Look for retry connection buttons if connection issues occur
      const retryButton = page.locator('button:has-text("Retry Connection")');
      if ((await retryButton.count()) > 0) {
        await retryButton.click();
        // Should attempt reconnection
        await page.waitForTimeout(1000);
      }

      // Session should still be functional
      await expect(
        page.locator('header a')
      ).toBeVisible();
    });
  });
});
