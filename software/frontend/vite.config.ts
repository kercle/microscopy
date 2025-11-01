import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// @ts-ignore (process isn't defined error)
const BACKEND_HOST = process.env.MICROSCOPE_BACKEND_HOST || 'localhost:3000';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		proxy: {
			'/api': { target: `http://${BACKEND_HOST}`, changeOrigin: true },
			'/api/ws': { target: `ws://${BACKEND_HOST}`, ws: true }
		}
	}
});
