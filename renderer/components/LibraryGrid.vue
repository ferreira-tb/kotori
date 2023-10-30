<script setup lang="ts">
import LibraryGridItem from '@/components/LibraryGridItem.vue';
import type { VisualNovel } from '@/database';

defineProps<{
	novels: VisualNovel[];
}>();
</script>

<template>
	<div class="kt-library-grid">
		<LibraryGridItem v-for="novel of novels" :key="novel.id" :novel="novel" />
	</div>
</template>

<style scoped>
.kt-library-grid {
	width: 100%;
	height: 100%;
	display: grid;
	grid-template-columns: repeat(auto-fill, 11rem);
	justify-content: center;
	gap: 0.5rem;
	overflow-x: hidden;
	overflow-y: auto;
}
</style>
