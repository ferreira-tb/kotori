<script setup lang="ts">
const store = useReaderStore();
const { book } = storeToRefs(store);
</script>

<template>
  <div>{{ book }}</div>
</template>
