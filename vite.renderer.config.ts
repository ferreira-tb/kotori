import { fileURLToPath, URL } from 'node:url';
import { builtinModules as builtin } from 'node:module';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import renderer from 'vite-plugin-electron-renderer';
import devTools from 'vite-plugin-vue-devtools';

export default defineConfig({
	plugins: [devTools(), vue(), renderer()],
	resolve: {
		alias: {
			'@': fileURLToPath(new URL('./renderer', import.meta.url))
		}
	},
	build: {
		target: 'esnext',
		chunkSizeWarningLimit: 5000,

		// https://sequelize.org/docs/v7/models/advanced/#caveat-with-minification
		minify: true,

		commonjsOptions: {
			ignoreDynamicRequires: true,
			strictRequires: 'auto',
			transformMixedEsModules: false
		},
		rollupOptions: {
			external: [
				/^@sequelize\/core/,
				'electron',
				...builtin,
				...builtin.map((m) => `node:${m}`)
			],
			input: 'index.html',
			output: {
				format: 'cjs',
				generatedCode: 'es2015'
			}
		}
	},
	define: {
		'process.env.NODE_ENV': '"production"'
	}
});
