import 'manatsu/style';
import './assets/style.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createManatsu } from 'manatsu';
import App from './App.vue';
import { router } from './router';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu({ darkMode: true });

app.use(router);
app.use(pinia);
app.use(manatsu);

router
  .push('/')
  .then(() => router.isReady())
  .then(() => app.mount('#app'))
  .catch((err: unknown) => console.error(err));

// When Tauri v2 is released, we should create a custom context menu.
globalThis.addEventListener('contextmenu', (e) => e.preventDefault());
