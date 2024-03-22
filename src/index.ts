import 'manatsu/components/style';
import '@manatsu/style/themes/mana';
import '@/assets/style.css';
import App from '@/App.vue';
import { createApp } from 'vue';
import { router } from '@/router';
import { createPinia } from 'pinia';
import { setupEventListeners } from '@/events';
import { createManatsu, registerComponents } from 'manatsu';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu({ darkMode: true });

app.use(router);
app.use(pinia);
app.use(manatsu);

registerComponents(app);
void setupEventListeners();

app.mount('#app');
