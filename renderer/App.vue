<script setup lang="ts">
import { RouterView } from 'vue-router';
import HeaderMenu from '@/components/HeaderMenu.vue';
</script>

<template>
	<main class="fixed inset-2">
		<header class="w-full">
			<HeaderMenu />
		</header>
		<div class="h-full w-full">
			<RouterView #default="{ Component }">
				<template v-if="Component">
					<Transition name="fade" mode="out-in">
						<Suspense>
							<component :is="Component" />
						</Suspense>
					</Transition>
				</template>
			</RouterView>
		</div>
	</main>
</template>
