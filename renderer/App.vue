<script setup lang="ts">
import { RouterView } from 'vue-router';
import HeaderMenu from '@/components/HeaderMenu.vue';
</script>

<template>
	<main class="kt-main">
		<header>
			<HeaderMenu />
		</header>
		<div>
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

<style scoped>
.kt-main {
	--kt-header-height: 2rem;

	display: flex;
	flex-direction: column;
	height: 100vh;
	width: 100vw;
}

.kt-main > header {
	flex-shrink: 0;
	flex-grow: 0;
	height: var(--kt-header-height);
}

.kt-main > div {
	height: calc(100% - var(--kt-header-height));
	width: 100%;
	overflow: hidden;
}
</style>
