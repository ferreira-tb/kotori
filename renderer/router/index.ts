import { createRouter, createMemoryHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';

export const router = createRouter({
	history: createMemoryHistory(),
	routes: [
		{
			path: '/',
			name: 'home',
			component: HomeView
		}
	]
});
