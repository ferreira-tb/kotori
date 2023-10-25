<script setup lang="ts">
import type { ResponsePostVisualNovel } from 'vndb-query';
import LibraryGridItem from '@/components/LibraryGridItem.vue';

defineProps<{
	works: ResponsePostVisualNovel[];
}>();
</script>

<template>
	<div class="grid grid-cols-6 p-1">
		<LibraryGridItem v-for="work of works" :key="work.id" :work="work" />
	</div>
</template>
