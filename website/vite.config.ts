import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
	base: "/cahnon/",
	server: {
		port: 3030,
		strictPort: true,
	},
	preview: {
		port: 3030,
		strictPort: true,
	},
	plugins: [tailwindcss()],
	build: {
		minify: "esbuild",
		cssCodeSplit: false,
		target: "esnext",
	},
});
