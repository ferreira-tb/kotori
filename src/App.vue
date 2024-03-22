<script setup lang="ts">
import { openFile } from '@/utils';
import type { SidebarItem } from 'manatsu';

const sidebarItems: SidebarItem[] = [{ key: 'all', label: 'All' }];
</script>

<template>
  <m-scaffold
    default-border="none"
    :sidebar-items
    sidebar-item-class="flex items-center justify-center"
  >
    <template #top>
      <m-top-appbar end-class="flex gap-4">
        <template #end>
          <m-button variant="outlined" @click="openFile">Open</m-button>
          <m-button variant="outlined">Add to Library</m-button>
        </template>
      </m-top-appbar>
    </template>

    <template #default>
      <router-view #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </router-view>
    </template>

    <template #sidebar-item="{ label }">
      <m-dynamic-link>{{ label }}</m-dynamic-link>
    </template>
  </m-scaffold>
</template>
