<script setup lang="ts">
import type { SidebarItem } from 'manatsu';

const sidebarItems: SidebarItem[] = [{ key: 'library', label: 'Library' }];

async function openFile() {
  const book = await invoke(Command.OpenFile);
  console.log(book);
}
</script>

<template>
  <m-scaffold
    default-border="none"
    :sidebar-items
    sidebar-item-class="flex items-center justify-center"
  >
    <template #top>
      <m-top-appbar start-class="flex gap-4">
        <template #start>
          <m-button variant="outlined" @click="openFile">Open</m-button>
          <m-button variant="outlined">Add to Library</m-button>
        </template>
      </m-top-appbar>
    </template>

    <template #default>
      <router-view #default="{ Component }">
        <template v-if="Component">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </template>
      </router-view>
    </template>

    <template #sidebar-item="{ label }">
      <m-dynamic-link>{{ label }}</m-dynamic-link>
    </template>
  </m-scaffold>
</template>
