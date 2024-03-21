<script setup lang="ts">
const store = useReaderStore();
const { book } = storeToRefs(store);
</script>

<template>
  <div>
    <div v-if="book">
      <img v-for="{ filename, path } of book.pages" :key="filename" :src="path" />
    </div>
  </div>
</template>
