import '@/assets/style.css';
import { createApp } from 'vue';
import { router } from '@/router';
import { sequelize } from '@/database';
import { RendererProcessError } from '@/utils/error';
import App from '@/App.vue';

sequelize.sync().then(async () => {
	const app = createApp(App);

	// Plugins.
	app.use(router);

	app.config.errorHandler = (err) => {
		RendererProcessError.catch(err);
	};

	await router.push('/');
	app.mount('#app');
});
