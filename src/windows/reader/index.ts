import '@/assets/style.css';
import 'manatsu/components/style';
import '@manatsu/style/themes/mana';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createManatsu, registerComponents } from 'manatsu';
import App from './App.vue';

const app = createApp(App);
const pinia = createPinia();
const manatsu = createManatsu({ darkMode: true });

app.use(pinia);
app.use(manatsu);

registerComponents(app);

app.mount('#app');
