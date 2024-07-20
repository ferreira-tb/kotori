import '@/assets/index.css';
import App from './App.vue';
import { createApp } from '@/lib/app';
import { setupEventListeners } from './events';

setupEventListeners();

createApp(App).mount('#app');
