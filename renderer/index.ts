import '@/assets/style.css';
import { createApp } from 'vue';
import { router } from '@/router';
import { sequelize } from '@/database';
import App from '@/App.vue';

sequelize.sync().then(async () => {
    const app = createApp(App);

    // Plugins.
    app.use(router);

    await router.push('/');
    app.mount('#app');
});