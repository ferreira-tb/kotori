<script setup lang="ts">
import { useCollectionStore } from '../stores';

const store = useCollectionStore();
const { collections } = storeToRefs(store);
</script>

<template>
  <div>
    <span v-for="collection of collections" :key="collection.id">
      {{ collection.name }}
    </span>
  </div>
</template>
