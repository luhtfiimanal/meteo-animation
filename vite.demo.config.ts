import { defineConfig } from 'vite';
import { existsSync } from 'fs';
import { join } from 'path';

export default defineConfig({
  root: 'demo',
  server: {
    port: 5180,
    open: true,
  },
  preview: {
    port: 5180,
  },
  appType: 'mpa',
  plugins: [
    {
      name: 'tiles-204',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          // Check if request is for a tile
          if (req.url?.startsWith('/tiles/')) {
            const filePath = join(__dirname, 'demo', req.url);
            if (!existsSync(filePath)) {
              // Return 204 No Content for missing tiles
              res.statusCode = 204;
              res.end();
              return;
            }
          }
          next();
        });
      },
    },
  ],
});
