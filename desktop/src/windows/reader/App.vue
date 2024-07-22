<script setup lang="ts">
import Reader from './views/Reader.vue';
import { useConfigStore } from '@/stores';
import { setSensors } from './lib/sensors';
import { showWindow } from '@/lib/commands';

setSensors();

onMounted(() => {
  useConfigStore().$tauri.start().then(flushPromises).then(showWindow).catch(handleError);
});
</script>

<template>
  <main class="fixed inset-0 select-none">
    <Reader />
  </main>
</template>
