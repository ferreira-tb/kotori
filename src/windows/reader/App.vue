<script setup lang="ts">
import Reader from './views/Reader.vue';
import { useReaderStore } from './stores';
import { setSensors } from './lib/sensors';
import { maximizeWindow, showWindow } from '@/lib/commands';

const store = useReaderStore();
const { ready } = storeToRefs(store);

setSensors();

onMounted(() => {
  until(ready)
    .toBeTruthy()
    .then(flushPromises)
    .then(maximizeWindow)
    .then(showWindow)
    .catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0 select-none">
    <Reader />
  </main>
</template>
