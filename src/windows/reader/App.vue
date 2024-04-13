<script setup lang="ts">
import { Command } from '@/utils/commands';
import { disableDefaultSensors, setGlobalSensors } from '@/utils/sensors';
import Reader from './views/Reader.vue';

disableDefaultSensors();
setGlobalSensors();

invokeOnKeyDown('Escape', Command.CloseCurrentWindow);
invokeOnKeyDown('F1', Command.FocusMainWindow);
</script>

<template>
  <reader />
</template>
