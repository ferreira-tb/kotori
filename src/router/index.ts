import { createMemoryHistory, createRouter } from 'vue-router';

export const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/HomeView.vue')
    }
  ]
});
