import { createRouter, createMemoryHistory } from 'vue-router';
import LibraryView from '@/views/LibraryView.vue';

export const router = createRouter({
	history: createMemoryHistory(),
	routes: [
		{
			path: '/',
			name: 'home',
			component: LibraryView
		}
	]
});
