<script setup lang="ts">
import { showWindow } from '@/lib/commands';
import Sidebar from './components/Sidebar.vue';
import { Button } from '@/components/ui/button';
import { setGlobalSensors } from '@/lib/sensors';
import { useConfigStore } from '@/stores/config';
import { Separator } from '@/components/ui/separator';
import { loadStores, useLibraryStore } from './stores';

const config = useConfigStore();
const { resume } = useIntervalFn(config.save, 30_000, { immediate: false });

const libraryStore = useLibraryStore();
const { library, selected } = storeToRefs(libraryStore);

setGlobalSensors();

onMounted(() => {
  loadStores().then(flushPromises).then(resume).then(showWindow).catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0 select-none">
    <div class="flex size-full flex-col overflow-hidden">
      <div class="flex flex-1">
        <Sidebar />
        <div class="w-full">
          <RouterView #default="{ Component }">
            <template v-if="Component">
              <component :is="Component" />
            </template>
          </RouterView>
        </div>
      </div>
      <Separator class="w-full" />
      <footer v-show="library.size > 0" class="flex flex-col overflow-hidden">
        <div class="flex h-16 items-center">
          <div v-if="selected" class="flex w-full justify-between px-2">
            <div class="flex items-center gap-2">
              <img v-if="selected.cover" :src="selected.cover" :alt="selected.title" class="h-10" />
              <div class="flex flex-col overflow-hidden">
                <div class="ellipsis">{{ selected.title }}</div>
                <div class="ellipsis text-xs">{{ selected.path }}</div>
              </div>
            </div>
            <div class="flex items-center pr-2">
              <Button class="h-8" @click="selected?.open()">Open</Button>
            </div>
          </div>
        </div>
      </footer>
    </div>
  </main>
</template>
