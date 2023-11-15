<script setup lang="ts">
import LibraryGridItem from '@/components/LibraryGridItem.vue';
import type { VisualNovel } from '@/database';

defineProps<{
    novels: VisualNovel[];
}>();
</script>

<template>
    <div class="kt-library-grid">
        <LibraryGridItem
            v-for="novel of novels"
            :key="novel.id"
            :novel="novel"
        />
    </div>
</template>

<style scoped>
.kt-library-grid {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fill, var(--kt-library-grid-item-width));
    justify-content: center;
    gap: var(--kt-library-grid-gap);
    overflow-x: hidden;
    overflow-y: auto;
}
</style>
