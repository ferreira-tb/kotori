<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useRoute } from 'vue-router';
import { useVisualNovel } from '@/composables';

const route = useRoute();
const routeId = toRef(() => route.params.id as string);
const routeName = toRef(() => route.name);

const id = computed(() => (routeName.value === 'vn' ? routeId.value : null));
const { novel } = useVisualNovel(id);
</script>

<template>
	<div>
		<div>User {{ $route.params.id }}</div>
		<div>{{ novel?.title }}</div>
	</div>
</template>
