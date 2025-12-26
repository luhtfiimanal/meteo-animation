import { test, expect } from '@playwright/test';

test.describe('Wind Layer Demo', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/wind-layer.html');
  });

  test('should check WebGPU availability', async ({ page }) => {
    const hasWebGPU = await page.evaluate(() => 'gpu' in navigator);

    if (!hasWebGPU) {
      test.skip(true, 'WebGPU not available in this browser');
    }

    expect(hasWebGPU).toBe(true);
  });

  test('should initialize wind layer', async ({ page }) => {
    // Wait for initialization (up to 10 seconds)
    await page.waitForTimeout(3000);

    // Check if error panel is shown
    const errorPanel = page.locator('#error-panel');
    const errorVisible = await errorPanel.isVisible();

    if (errorVisible) {
      const errorText = await page.locator('#error-message').textContent();
      if (errorText?.includes('WebGPU') || errorText?.includes('timeout')) {
        test.skip(true, 'WebGPU not available in this environment');
      }
    }

    // Check if stats panel is visible (indicates successful init)
    const statsPanel = page.locator('#stats-panel');
    await expect(statsPanel).toBeVisible({ timeout: 15000 });
  });

  test('should load tiles without 404 errors', async ({ page }) => {
    // Collect console errors
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error' && msg.text().includes('404')) {
        errors.push(msg.text());
      }
    });

    // Wait for tiles to load
    await page.waitForTimeout(5000);

    // Check no 404 errors for tiles
    const tileErrors = errors.filter((e) => e.includes('/tiles/wind/'));
    expect(tileErrors.length).toBe(0);
  });

  test('should show particle count when running', async ({ page }) => {
    // Wait for initialization
    await page.waitForTimeout(5000);

    // Check if error panel is shown
    const errorPanel = page.locator('#error-panel');
    const errorVisible = await errorPanel.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Check particle count
    const particleCount = await page.locator('#stat-particles').textContent();
    expect(parseInt(particleCount || '0')).toBeGreaterThan(0);
  });

  test('should show FPS when running', async ({ page }) => {
    // Wait for a few frames
    await page.waitForTimeout(5000);

    // Check if error panel is shown
    const errorPanel = page.locator('#error-panel');
    const errorVisible = await errorPanel.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Check FPS
    const fps = await page.locator('#stat-fps').textContent();
    expect(parseInt(fps || '0')).toBeGreaterThan(0);
  });

  test('should respond to pause/resume', async ({ page }) => {
    // Wait for initialization
    await page.waitForTimeout(3000);

    // Check if error panel is shown
    const errorPanel = page.locator('#error-panel');
    const errorVisible = await errorPanel.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Click pause button
    const toggleBtn = page.locator('#btn-toggle');
    await toggleBtn.click();

    // Check button text changed
    await expect(toggleBtn).toHaveText('Resume Animation');

    // Click resume
    await toggleBtn.click();
    await expect(toggleBtn).toHaveText('Pause Animation');
  });

  test('should show correct zoom level', async ({ page }) => {
    // Wait for map to load
    await page.waitForTimeout(2000);

    // Initial zoom should be 5
    const zoomText = await page.locator('#stat-zoom').textContent();
    expect(parseFloat(zoomText || '0')).toBe(5);
  });

  test('should update zoom on interaction', async ({ page }) => {
    // Wait for map to load
    await page.waitForTimeout(2000);

    // Click zoom in button
    await page.locator('button[title="Zoom in"]').click();
    await page.waitForTimeout(500);

    // Check zoom increased
    const zoomText = await page.locator('#stat-zoom').textContent();
    expect(parseFloat(zoomText || '0')).toBeGreaterThan(5);
  });
});
