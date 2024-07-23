import { RouteName } from './routes';
import { LibraryMode } from './query';
import { createMemoryHistory, createRouter, type LocationQueryRaw } from 'vue-router';

const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: '/',
      name: RouteName.Library,
      component: () => import('../views/Library.vue'),
    },
    {
      path: '/collection',
      name: RouteName.Collection,
      component: () => import('../views/Collection.vue'),
    },
    {
      path: '/tag',
      name: RouteName.BookTag,
      component: () => import('../views/BookTag.vue'),
    },
  ],
});

function navigate(name: RouteName, query?: LocationQueryRaw) {
  void router.push({ name, query });
}

function navigateToLibrary(query?: LocationQueryRaw) {
  navigate(RouteName.Library, query);
}

function navigateToTags() {
  navigate(RouteName.BookTag);
}

export { LibraryMode, navigate, navigateToLibrary, navigateToTags, RouteName, router };
