import react from '@vitejs/plugin-react';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import { fumadocsMdx } from 'fumadocs-mdx/vite';

export default defineConfig({
  server: {
    port: 3000,
  },
  plugins: [
    fumadocsMdx(),
    tailwindcss(),
    tanstackStart({
      spa: {
        enabled: true,
        prerender: {
          enabled: true,
          outputPath: '/index',
          crawlLinks: true,
        },
      },
      pages: [
        {
          path: '/docs',
        },
        {
          path: '/docs/cli',
        },
        {
          path: '/docs/python',
        },
        {
          path: '/api/search',
        },
        {
          path: '/llms-full.txt',
        },
        {
          path: '/llms.txt',
        },
      ],
    }),
    react(),
  ],
  resolve: {
    tsconfigPaths: true,
    alias: {
      tslib: 'tslib/tslib.es6.js',
    },
  },
});
