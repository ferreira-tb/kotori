<script setup lang="ts">
import { Command } from '@/utils/commands';

invokeOnKeyDown('Tab', Command.SwitchReaderFocus);

invokeOnKeyDown('o', Command.OpenBookFromDialog, null, { ctrlKey: true });
invokeOnKeyDown('A', Command.AddToLibraryFromDialog, null, { ctrlKey: true, shiftKey: true });

preventContextMenu();
</script>

<template>
  <main>
    <router-view #default="{ Component }">
      <template v-if="Component">
        <component :is="Component" />
      </template>
    </router-view>
  </main>
</template>
