<script setup lang="ts">
import { symbols } from './lib/symbols';
import { toPixel } from '@tb-dev/utils';
import { useLibraryStore } from './stores';
import { setSensors } from './lib/sensors';
import { showWindow } from '@/lib/commands';
import Navbar from './components/Navbar.vue';

const navbar = ref<HTMLElement>();
const navbarHeight = useHeight(navbar);
const windowHeight = useWindowHeight();
provide(symbols.windowHeight, windowHeight);

const contentHeight = computed(() => windowHeight.value - navbarHeight.value);
provide(symbols.contentHeight, contentHeight);

setSensors();

onMounted(() => {
  useLibraryStore().load().then(flushPromises).then(showWindow).catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0 select-none">
    <div ref="navbar" class="absolute inset-x-0 top-0">
      <Navbar />
    </div>

    <div
      class="relative w-full overflow-hidden px-2 pb-2 pt-0"
      :style="{ top: toPixel(navbarHeight), height: contentHeight }"
    >
      <RouterView #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </RouterView>
    </div>
  </main>
</template>
