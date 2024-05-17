import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import { createManatsu } from 'manatsu';
import { createApp as createVue } from 'vue';

export function createApp(root: Component) {
  const app = createVue(root);
  const pinia = createPinia();
  const manatsu = createManatsu();

  app.use(pinia);
  app.use(manatsu);
  app.use(PrimeVue);

  return app;
}
