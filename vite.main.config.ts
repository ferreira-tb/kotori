import { builtinModules as builtin } from 'node:module';
import { defineConfig } from 'vite';

export default defineConfig({
	resolve: {
		browserField: false,
		conditions: ['node'],
		mainFields: ['module', 'jsnext:main', 'jsnext']
	},
	build: {
		target: 'esnext',
		chunkSizeWarningLimit: 5000,
		minify: true,
		rollupOptions: {
			external: [
				'electron',
				...builtin,
				...builtin.map((m) => `node:${m}`)
			],
			input: 'main/index.ts',
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
