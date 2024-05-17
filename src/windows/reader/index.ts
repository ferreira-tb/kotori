import '@/lib/theme';
import App from './App.vue';
import { createApp } from '@/lib/app';
import { setupEventListeners } from './events';

setupEventListeners();

createApp(App).mount('#app');
