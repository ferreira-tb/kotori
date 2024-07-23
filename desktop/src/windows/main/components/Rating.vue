<script setup lang="ts">
import { cn } from '@/lib/utils';
import { Star } from 'lucide-vue-next';

defineProps<{ rating: number; }>();
defineEmits<(e: 'update', rating: number) => void>();
</script>

<template>
  <div class="flex gap-1">
    <Star
      v-for="amount of 5"
      :key="amount"
      :fill="amount <= rating ? 'currentColor' : 'none'"
      :class="cn('size-4 cursor-pointer transition-all', amount > rating && 'hover:scale-105')"
      @click="$emit('update', amount)"
    />
  </div>
</template>
