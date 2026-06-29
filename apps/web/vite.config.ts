import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  optimizeDeps: {
    include: ['react-router-dom', 'urql', 'recharts', 'lucide-react'],
  },
  server: {
    port: 5173,
    proxy: {
      '/graphql': 'http://localhost:8000',
      '/health': 'http://localhost:8000',
    },
  },
  preview: {
    proxy: {
      '/graphql': 'http://localhost:8000',
      '/health': 'http://localhost:8000',
    },
  },
})
