<script setup lang="ts">
const pages = computed(() => []);

const { /*state: current,*/ prev, next, go } = useCycleList(pages);
onKeyDown('ArrowLeft', () => prev());
onKeyDown('ArrowRight', () => next());
onKeyDown('Home', () => go(0));
onKeyDown('End', () => go(pages.value.length - 1));

const id = window.__KOTORI__.readerId;
</script>

<template>
  <m-scaffold>
    <div class="flex size-full items-center justify-center">
      <div class="text-2xl">{{ id }}</div>
      <div class="flex size-full flex-col items-center justify-center overflow-hidden">
        <img class="size-full object-contain" />
      </div>
    </div>
  </m-scaffold>
</template>
