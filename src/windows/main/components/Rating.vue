<script setup lang="ts">
import { cn } from '@/lib/utils';
import { Star } from 'lucide-vue-next';

const rating = ref(3);

function update(value: number) {
  rating.value = value;
}
</script>

<template>
  <div class="flex gap-1">
    <Star
      v-for="idx of 5"
      :key="idx"
      :fill="idx <= rating ? 'currentColor' : 'none'"
      :class="cn('h-4 w-4 cursor-pointer transition-all', idx > rating && 'hover:scale-105')"
      @click="update(idx)"
    />
  </div>
</template>
