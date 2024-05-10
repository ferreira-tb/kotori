<script setup lang="ts">
import Reader from './views/Reader.vue';
import { setGlobalSensors } from '@/lib/sensors';
import { closeWindow, focusMainWindow } from '@/lib/commands';

setGlobalSensors();

onKeyDown('Escape', closeWindow);
onKeyDown('F1', focusMainWindow);
</script>

<template>
  <reader />
</template>
