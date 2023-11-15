import { createRouter, createMemoryHistory } from 'vue-router';
import LibraryView from '@/views/LibraryView.vue';
import VisualNovelView from '@/views/VisualNovelView.vue';

export const router = createRouter({
    history: createMemoryHistory(),
    routes: [
        {
            path: '/',
            name: 'home',
            component: LibraryView
        },
        {
            path: '/vn/:id',
            name: 'vn',
            component: VisualNovelView
        }
    ]
});
