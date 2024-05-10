<script setup lang="ts">
import { RouteName } from './router';
import { toPixel } from '@tb-dev/utils';
import { symbols } from './lib/symbols';
import { useLibraryStore } from './stores';
import { showWindow } from '@/lib/commands';
import { setGlobalSensors } from '@/lib/sensors';
import type { MenuItem } from 'primevue/menuitem';

const menubar = ref<HTMLElement | null>(null);
const { height: menubarHeight } = useElementSize(menubar);
const { height: windowHeight } = useWindowSize();

const contentHeight = computed(() => toPixel(windowHeight.value - menubarHeight.value));
provide(symbols.contentHeight, contentHeight);

const router = useRouter();
const menuItems: MenuItem[] = [
  { label: 'Library', command: () => void router.push({ name: RouteName.Library }) },
  { label: 'Collection', command: () => void router.push({ name: RouteName.Collection }) },
  { label: 'Tags', command: () => void router.push({ name: RouteName.BookTag }) }
];

setGlobalSensors();

onMounted(() => {
  const library = useLibraryStore();
  library.load().then(flushPromises).then(showWindow).catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0">
    <div ref="menubar" class="absolute inset-x-0 top-0">
      <p-menubar :model="menuItems" class="rounded-none border-none">
        <template #end>
          <div id="kt-menubar-end"></div>
        </template>
      </p-menubar>
    </div>

    <div v-if="menubar" id="kt-content">
      <router-view #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </router-view>
    </div>
  </main>
</template>

<style scoped>
#kt-content {
  position: relative;
  top: v-bind('toPixel(menubarHeight)');
  padding: 0 0.5rem 0.5rem;
  width: 100%;
  height: v-bind('contentHeight');
  overflow: hidden;
}
</style>
