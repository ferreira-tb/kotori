import '@/lib/theme';
import App from './App.vue';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import { RouteName, router } from './router';
import { setupEventListeners } from './events';
import { createManatsu, handleError } from 'manatsu';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu();

app.use(router);
app.use(pinia);
app.use(manatsu);
app.use(PrimeVue);

setupEventListeners();

router
  .push({ name: RouteName.Library })
  .then(() => app.mount('#app'))
  .catch(handleError);
