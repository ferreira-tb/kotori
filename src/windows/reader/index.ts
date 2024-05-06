import '@/assets/style.css';
import 'primevue/resources/themes/aura-dark-noir/theme.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import { createManatsu } from 'manatsu';
import App from './App.vue';
import { setupEventListeners } from './events';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu();

app.use(pinia);
app.use(manatsu);
app.use(PrimeVue);

setupEventListeners();

app.mount('#app');
