<script setup lang="ts">
import { Command } from '@/utils/commands';

invokeOnKeyDown('Tab', Command.SwitchReaderFocus);
invokeOnKeyDown('o', Command.OpenBookFromDialog, null, { ctrlKey: true });
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
