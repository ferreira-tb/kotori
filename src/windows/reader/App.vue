<script setup lang="ts">
import { Command } from '@/utils/commands';
import Reader from './views/Reader.vue';

preventContextMenu();
preventKeyDown(['F3', 'F7']);
invokeOnKeyDown('Escape', Command.CloseCurrentWindow);
invokeOnKeyDown('Tab', Command.SwitchReaderFocus);
invokeOnKeyDown('F1', Command.FocusMainWindow);
invokeOnKeyDown('o', Command.OpenBookFromDialog, null, { ctrlKey: true });
invokeOnKeyDown('A', Command.AddToLibraryFromDialog, null, { ctrlKey: true, shiftKey: true });
</script>

<template>
  <reader />
</template>
