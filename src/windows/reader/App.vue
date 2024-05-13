<script setup lang="ts">
import Reader from './views/Reader.vue';
import { useReaderStore } from './stores';
import { setGlobalSensors } from '@/lib/sensors';
import { closeWindow, focusMainWindow, maximizeWindow, showWindow } from '@/lib/commands';

const store = useReaderStore();
const { ready } = storeToRefs(store);

setGlobalSensors();

onKeyDown('Escape', closeWindow);
onKeyDown('F1', focusMainWindow);

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
  <reader />
</template>
