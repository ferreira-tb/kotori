<script setup lang="ts">
import type { ResponsePostVisualNovel } from 'vndb-query';

defineProps<{
	work: ResponsePostVisualNovel;
}>();
</script>

<template>
	<img :src="work.image!.url!" />
</template>
