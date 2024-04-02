<script setup lang="ts">
const pages = computed(() => []);

const { /*state: current,*/ prev, next, go } = useCycleList(pages);
onKeyDown('ArrowLeft', () => prev());
onKeyDown('ArrowRight', () => next());
onKeyDown('Home', () => go(0));
onKeyDown('End', () => go(pages.value.length - 1));
</script>

<template>
  <m-scaffold>
    <div class="flex size-full items-center justify-center">
      <div class="flex size-full flex-col items-center justify-center overflow-hidden">
        <img class="size-full object-contain" />
      </div>
    </div>
  </m-scaffold>
</template>
