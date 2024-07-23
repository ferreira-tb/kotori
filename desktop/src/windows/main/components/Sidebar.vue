<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Library, Plus, Star, Tag } from 'lucide-vue-next';
import { LibraryMode, navigateToLibrary, navigateToTags } from '../router';
</script>

<template>
  <nav class="hidden h-full w-60 space-y-4 py-2 lg:block">
    <div class="px-3 py-2">
      <h2 class="mb-2 px-4 text-lg font-semibold tracking-tight">Library</h2>
      <div class="space-y-1">
        <Button variant="ghost" class="w-full justify-start" @click="navigateToLibrary()">
          <Library class="mr-2 size-4" />
          <span>Books</span>
        </Button>
        <Button
          variant="ghost"
          class="w-full justify-start"
          @click="navigateToLibrary({ mode: LibraryMode.Favorites })"
        >
          <Star class="mr-2 size-4" />
          <span>Favorites</span>
        </Button>
        <Button variant="ghost" class="w-full justify-start" @click="navigateToTags()">
          <Tag class="mr-2 size-4" />
          <span>Tags</span>
        </Button>
      </div>
    </div>
    <div class="px-3 py-2">
      <h2 class="mb-2 px-4 text-lg font-semibold tracking-tight">Collections</h2>
      <div class="flex items-center justify-center">
        <Button variant="outline">
          <Plus class="mr-2 size-4" />
          <span>New collection</span>
        </Button>
      </div>
    </div>
  </nav>
</template>
