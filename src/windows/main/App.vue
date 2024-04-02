<script setup lang="ts">
import { RouteName } from './router';

navigateOnKeyDown('F1', { name: RouteName.Library });
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
