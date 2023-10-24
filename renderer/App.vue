<script setup lang="ts">
import { RouterView } from 'vue-router';
import TheMainMenu from '@/components/TheMainMenu.vue';
</script>

<template>
	<main class="fixed inset-2">
		<div>
			<TheMainMenu />
		</div>
		<div class="w-full">
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
