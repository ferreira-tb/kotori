<script setup lang="ts">
import LibraryGridItem from '@/components/LibraryGridItem.vue';
import type { VisualNovel } from '@/database';

defineProps<{
	novels: VisualNovel[];
}>();
</script>

<template>
	<div class="grid grid-cols-6 p-1">
		<LibraryGridItem
			v-for="novel of novels"
			:key="novel.id"
			:novel="novel"
		/>
	</div>
</template>
