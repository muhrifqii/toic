/// <reference types="vitest/config" />

import { fileURLToPath, URL } from 'url'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import type { ViteUserConfig as VitestUserConfigInterface } from 'vitest/config'
import dotenv from 'dotenv'
import environment from 'vite-plugin-environment'

dotenv.config({ path: '../../.env' })

const vitestConfig: VitestUserConfigInterface = {
  test: {
    environment: 'jsdom',
    globals: true,
    coverage: {
      provider: 'v8',
      reportsDirectory: '../../coverage/fe',
      reporter: ['html', 'cobertura']
    }
  }
}

export default defineConfig({
  build: {
    emptyOutDir: true
  },
  test: vitestConfig.test,
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis'
      }
    }
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:4943',
        changeOrigin: true
      }
    }
  },
  plugins: [
    react(),
    tailwindcss(),
    environment('all', { prefix: 'CANISTER_' }),
    environment('all', { prefix: 'DFX_' })
  ],
  resolve: {
    alias: [
      {
        find: '@declarations',
        replacement: fileURLToPath(new URL('../declarations', import.meta.url))
      },
      {
        find: '@',
        replacement: fileURLToPath(new URL('./src', import.meta.url))
      }
    ],
    dedupe: ['@dfinity/agent']
  },
  envDir: '../..',
  envPrefix: ['DFX_', 'CANISTER_']
})
