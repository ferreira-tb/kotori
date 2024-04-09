<script setup lang="ts">
import { useLibraryStore } from '../stores';

const libraryStore = useLibraryStore();
</script>

<template>
  <div>
    <p-input-text
      v-model="libraryStore.filter"
      size="small"
      placeholder="Search"
      spellcheck="false"
    />
  </div>
</template>
