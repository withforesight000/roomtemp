import { test, expect } from "@playwright/test";
import { installTauriMocks } from "./fixtures/mock-tauri";

test.beforeEach(async ({ page }) => {
  await installTauriMocks(page);
});

test("renders connection status and navigation", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("header p")).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('a[href="/settings"]')).toBeVisible();
  await expect(page.locator('a[href="/"]')).toBeVisible();
});

test("updates settings and toggles proxy", async ({ page }) => {
  await page.goto("/settings");

  const urlInput = page.getByPlaceholder("https://example.com/grpc");
  await expect(urlInput).toBeVisible();

  const accessTokenInput = page.getByPlaceholder("Your Access Token");
  await accessTokenInput.fill("another-token");

  const proxyCheckbox = page.getByRole("checkbox", {
    name: "Use a Proxy Server",
  });
  const proxyInput = page.getByPlaceholder("http://proxy.example.com:8080");

  await expect(proxyInput).toBeDisabled();
  await proxyCheckbox.click();
  await expect(proxyInput).toBeEnabled();
  await proxyInput.fill("http://proxy.local:8080");

  await page.getByRole("button", { name: "Update" }).click();
});

test("fetch graph data shows charts", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Fetch Data" }).click();

  await expect(page.getByText("Temperature (℃)")).toBeVisible();
  await expect(page.getByText("Humidity (%)")).toBeVisible();
  await expect(page.getByText("Illumination (lx)")).toBeVisible();
});
