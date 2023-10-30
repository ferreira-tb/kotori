<script setup lang="ts">
const emit = defineEmits<{
	(e: 'search', value: string): void;
}>();

function search(e: Event) {
	const value = (e.currentTarget as HTMLInputElement | null)?.value;
	if (typeof value === 'string' && value.length > 0) emit('search', value);
}
</script>

<template>
	<div>
		<input type="text" placeholder="Search" @change="search" />
	</div>
</template>
