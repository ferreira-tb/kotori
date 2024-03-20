<script setup lang="ts">
import { MScaffold } from 'manatsu';
</script>

<template>
  <m-scaffold default-border="none">
    <template #default>
      <router-view #default="{ Component }">
        <template v-if="Component">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </template>
      </router-view>
    </template>
  </m-scaffold>
</template>
