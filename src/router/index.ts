import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/Home.vue')
    },
    {
      path: '/reader',
      name: 'reader',
      component: () => import('../views/Reader.vue')
    }
  ]
});

export { router };
