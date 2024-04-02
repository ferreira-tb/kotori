import '@/assets/style.css';
import 'manatsu/components/style';
import '@manatsu/style/themes/mana';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { emit } from '@tauri-apps/api/event';
import { createManatsu, handleError, registerComponents } from 'manatsu';
import App from './App.vue';
import { Event } from './events';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu({ darkMode: true });

app.use(pinia);
app.use(manatsu);

registerComponents(app);

emit(Event.WillMountReader)
  .then(() => app.mount('#app'))
  .catch(handleError);
