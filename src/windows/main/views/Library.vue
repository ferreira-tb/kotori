<script setup lang="ts">
import { useLibraryStore } from '../stores';

const store = useLibraryStore();
const { books } = storeToRefs(store);

watchEffect(() => {
  console.log('Books:', books.value);
});
</script>

<template>
  <main></main>
</template>
