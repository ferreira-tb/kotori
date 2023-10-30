<script setup lang="ts">
import { getVisualNovelsByTitle } from '@/utils/query';

function search(e: Event) {
	const title = (e.currentTarget as HTMLInputElement | null)?.value;
	if (typeof title !== 'string' || title.length === 0) return;
	getVisualNovelsByTitle(title);
}
</script>

<template>
	<div>
		<input type="text" placeholder="Search" @change="search" />
	</div>
</template>
