import { createMemoryHistory, createRouter } from 'vue-router';
import { RouteName } from './routes';

const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: '/',
      name: RouteName.Library,
      component: () => import('../views/Library.vue')
    }
  ]
});

export { RouteName, router };
