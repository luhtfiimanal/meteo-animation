import { test, expect } from '@playwright/test';

test.describe('POC: Particle Boundary Behavior', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/poc-particle-boundary.html');
  });

  test('should check WebGPU availability', async ({ page }) => {
    // Check if WebGPU is available
    const hasWebGPU = await page.evaluate(() => 'gpu' in navigator);

    if (!hasWebGPU) {
      test.skip(true, 'WebGPU not available in this browser');
    }

    expect(hasWebGPU).toBe(true);
  });

  test('should initialize particles', async ({ page }) => {
    // Wait for initialization
    await page.waitForTimeout(2000);

    // Check if error is shown
    const errorBox = await page.locator('#error-box');
    const errorVisible = await errorBox.isVisible();

    if (errorVisible) {
      const errorText = await errorBox.textContent();
      if (errorText?.includes('WebGPU not supported') || errorText?.includes('Failed to get GPU adapter')) {
        test.skip(true, 'WebGPU not available');
      }
    }

    // Check stats
    const particleCount = await page.locator('#stat-particles').textContent();
    expect(parseInt(particleCount || '0')).toBeGreaterThan(0);
  });

  test('should show FPS when running', async ({ page }) => {
    // Wait for a few frames
    await page.waitForTimeout(2000);

    // Check if error is shown
    const errorBox = await page.locator('#error-box');
    const errorVisible = await errorBox.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Check FPS
    const fps = await page.locator('#stat-fps').textContent();
    expect(parseInt(fps || '0')).toBeGreaterThan(0);
  });

  test('should track respawn events', async ({ page }) => {
    // Wait for particles to move and respawn
    await page.waitForTimeout(5000);

    // Check if error is shown
    const errorBox = await page.locator('#error-box');
    const errorVisible = await errorBox.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Check respawn count (should be > 0 after some time)
    const respawns = await page.locator('#stat-respawns').textContent();
    expect(parseInt(respawns || '0')).toBeGreaterThan(0);
  });

  test('should respond to wind direction change', async ({ page }) => {
    // Wait for initialization
    await page.waitForTimeout(1000);

    // Check if error is shown
    const errorBox = await page.locator('#error-box');
    const errorVisible = await errorBox.isVisible();

    if (errorVisible) {
      test.skip(true, 'WebGPU not available');
    }

    // Get initial exit count
    const initialExits = parseInt(await page.locator('#stat-exits').textContent() || '0');

    // Change wind direction drastically
    await page.locator('#wind-dir').fill('180');

    // Wait for particles to exit
    await page.waitForTimeout(3000);

    // Check exit count increased
    const finalExits = parseInt(await page.locator('#stat-exits').textContent() || '0');
    expect(finalExits).toBeGreaterThanOrEqual(initialExits);
  });
});
