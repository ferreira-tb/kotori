import '@/assets/style.css';
import 'primevue/resources/themes/aura-dark-noir/theme.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import { createManatsu } from 'manatsu';
import App from './App.vue';
import { useReaderStore } from './stores';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu();

app.use(pinia);
app.use(manatsu);
app.use(PrimeVue);

const store = useReaderStore();
const { readerId } = storeToRefs(store);

until(readerId)
  .toMatch((id) => typeof id === 'number', { timeout: 5000, throwOnTimeout: true })
  .then(() => app.mount('#app'))
  .catch(handleError);
