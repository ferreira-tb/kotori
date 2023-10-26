<script setup lang="ts">
import type { VisualNovel } from '@/database';

defineProps<{
	novel: VisualNovel;
}>();
</script>

<template>
	<div>
		<img :src="novel.image!.url" class="h-96 w-48 object-cover" />
	</div>
</template>
