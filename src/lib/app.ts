import 'es-iterator-helpers/Iterator.prototype.some/auto';
import 'es-iterator-helpers/Iterator.prototype.find/auto';
import 'es-iterator-helpers/Iterator.prototype.every/auto';
import 'es-iterator-helpers/Iterator.prototype.toArray/auto';
import { createPinia } from 'pinia';
import { createApp as createVue } from 'vue';
import { createPlugin } from 'tauri-plugin-pinia';
import { createManatsu, handleError } from 'manatsu';

export function createApp(root: Component) {
  const app = createVue(root);
  const pinia = createPinia();
  const manatsu = createManatsu();

  pinia.use(createPlugin({ onError: handleError }));

  app.use(pinia);
  app.use(manatsu);

  return app;
}
