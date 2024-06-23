<script setup lang="ts">
import Reader from './views/Reader.vue';
import { useReaderStore } from './stores';
import { useConfigStore } from '@/stores';
import { setSensors } from './lib/sensors';
import { showWindow } from '@/lib/commands';

const store = useReaderStore();
const { ready } = storeToRefs(store);

setSensors();

onMounted(() => {
  const config = useConfigStore();
  const promises = [config.load(), until(ready).toBeTruthy()];
  Promise.all(promises).then(flushPromises).then(showWindow).catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0 select-none">
    <Reader />
  </main>
</template>
