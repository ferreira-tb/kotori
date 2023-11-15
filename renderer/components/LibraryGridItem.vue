<script setup lang="ts">
import { computed } from 'vue';
import { router } from '@/router';
import type { VisualNovel } from '@/database';

const props = defineProps<{
    novel: VisualNovel;
}>();

const url = computed(() => `url("${props.novel.image!.url}")`);

function onClick() {
    router.push({ name: 'vn', params: { id: props.novel.id } });
}
</script>

<template>
    <div class="kt-library-grid-item" @click="onClick"></div>
</template>

<style scoped>
.kt-library-grid-item {
    width: var(--kt-library-grid-item-width);
    height: var(--kt-library-grid-item-height);
    overflow: hidden;
    background-size: cover;
    background-repeat: no-repeat;
    background-image: v-bind('url');
    background-position: center;
    border-radius: var(--kt-library-grid-item-border-radius);
    cursor: pointer;
}

.kt-library-grid-item:hover {
    filter: brightness(0.8);
    transition: filter 0.2s ease-in-out;
}
</style>
