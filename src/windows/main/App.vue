<script setup lang="ts">
import { Command } from '@/utils/commands';

invokeOnKeyDown('o', Command.OpenBookFromDialog, null, { ctrlKey: true });
</script>

<template>
  <m-scaffold default-border="none">
    <template #default>
      <router-view #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </router-view>
    </template>
  </m-scaffold>
</template>
