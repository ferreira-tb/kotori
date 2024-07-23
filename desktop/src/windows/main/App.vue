<script setup lang="ts">
import { showWindow } from '@/lib/commands';
import Rating from './components/Rating.vue';
import Sidebar from './components/Sidebar.vue';
import { Button } from '@/components/ui/button';
import { setGlobalSensors } from '@/lib/sensors';
import { Separator } from '@/components/ui/separator';
import { loadStores, useLibraryStore } from './stores';

const libraryStore = useLibraryStore();
const { library, selected } = storeToRefs(libraryStore);

setGlobalSensors();

onMounted(() => {
  loadStores()
    .then(flushPromises)
    .then(showWindow)
    .catch(handleError);
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
      <footer v-show="library.size > 0 && selected" class="flex flex-col overflow-hidden">
        <div class="flex h-16 items-center">
          <div v-if="selected" class="flex w-full justify-between px-2">
            <div class="flex items-center gap-2">
              <img v-if="selected.cover" :src="selected.cover" :alt="selected.title" class="h-10">
              <div class="flex flex-col gap-1 overflow-hidden">
                <div class="ellipsis">{{ selected.title }}</div>
                <Rating />
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
