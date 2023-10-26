<script setup lang="ts">
import type { VisualNovel } from '@/database';

defineProps<{
	novel: VisualNovel;
}>();
</script>

<template>
	<img :src="novel.image!.url" />
</template>
