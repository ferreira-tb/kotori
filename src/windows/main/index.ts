import '@/assets/index.css';
import App from './App.vue';
import { createApp } from '@/lib/app';
import { handleError } from 'manatsu';
import { RouteName, router } from './router';
import { setupEventListeners } from './events';

setupEventListeners();

const app = createApp(App).use(router);

router
  .push({ name: RouteName.Library })
  .then(() => app.mount('#app'))
  .catch(handleError);
