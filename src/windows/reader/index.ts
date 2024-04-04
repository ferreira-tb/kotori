import '@/assets/style.css';
import 'manatsu/components/style';
import '@manatsu/style/themes/mana';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createManatsu, registerComponents } from 'manatsu';
import App from './App.vue';
import { useReaderStore } from './stores';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu({ darkMode: true });

app.use(pinia);
app.use(manatsu);

registerComponents(app);

const store = useReaderStore();
const { readerId } = storeToRefs(store);

until(readerId)
  .toMatch((id) => typeof id === 'number', { timeout: 5000, throwOnTimeout: true })
  .then(() => app.mount('#app'))
  .catch(handleError);
