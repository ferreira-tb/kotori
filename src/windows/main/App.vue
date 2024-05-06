<script setup lang="ts">
import { toPixel } from '@tb-dev/utils';
import { showWindow } from '@/utils/commands';
import type { MenuItem } from 'primevue/menuitem';
import { disableDefaultSensors, setGlobalSensors } from '@/utils/sensors';
import { RouteName } from './router';
import { symbols } from './utils/symbols';
import LibraryMenu from './components/LibraryMenu.vue';

const route = useRoute();
const router = useRouter();

const menubar = shallowRef<HTMLElement | null>(null);
const { height: menubarHeight } = useElementSize(menubar);
const { height: windowHeight } = useWindowSize();

const contentHeight = computed(() => toPixel(windowHeight.value - menubarHeight.value));
provide(symbols.contentHeight, contentHeight);

const menuItems: MenuItem[] = [
  { label: 'Library', command: () => void router.push({ name: RouteName.Library }) },
  { label: 'Collection', command: () => void router.push({ name: RouteName.Collection }) },
  { label: 'Tags', command: () => void router.push({ name: RouteName.BookTag }) }
];

disableDefaultSensors();
setGlobalSensors();

onMounted(() => showWindow().catch(handleError));
</script>

<template>
  <main class="fixed inset-0">
    <div ref="menubar" class="absolute inset-x-0 top-0">
      <p-menubar :model="menuItems" class="rounded-none border-none">
        <template #end>
          <library-menu v-if="route.name === RouteName.Library" />
        </template>
      </p-menubar>
    </div>

    <div id="app-content">
      <router-view #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </router-view>
    </div>
  </main>
</template>

<style scoped>
#app-content {
  position: relative;
  top: v-bind('toPixel(menubarHeight)');
  padding: 0 0.5rem 0.5rem;
  width: 100%;
  height: v-bind('contentHeight');
  overflow: hidden;
}
</style>
