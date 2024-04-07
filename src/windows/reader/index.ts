import 'primevue/resources/themes/aura-dark-noir/theme.css';
import '@/assets/style.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import App from './App.vue';
import { useReaderStore } from './stores';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(PrimeVue);

const store = useReaderStore();
const { readerId } = storeToRefs(store);

until(readerId)
  .toMatch((id) => typeof id === 'number', { timeout: 5000, throwOnTimeout: true })
  .then(() => app.mount('#app'))
  .catch(handleError);
