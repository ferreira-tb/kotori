import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'library',
      component: () => import('../views/Library.vue')
    },
    {
      path: '/reader',
      name: 'reader',
      component: () => import('../views/Reader.vue')
    }
  ]
});

export { router };
