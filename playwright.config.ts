import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './test',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:5180',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium-webgpu',
      use: {
        ...devices['Desktop Chrome'],
        channel: 'chrome',  // Use installed Chrome instead of bundled Chromium
        launchOptions: {
          args: [
            '--enable-unsafe-webgpu',
            '--enable-features=Vulkan',
            '--use-angle=vulkan',
            '--enable-gpu-rasterization',
            '--ignore-gpu-blocklist',
          ],
        },
      },
    },
  ],

  webServer: {
    command: 'bun run demo',
    url: 'http://localhost:5180',
    reuseExistingServer: !process.env.CI,
  },
});
