<script setup lang="ts">
import { Command } from '@/utils/commands';
import { Event } from '../../events';

const visible = ref(false);
const book = ref<number | null>(null);
const title = ref<string | null>(null);
const loading = ref(false);

useListen<RemoveBookRequestedPayload>(Event.RemoveBookRequested, ({ payload }) => {
  book.value = payload.id;
  title.value = payload.title;
  visible.value = true;
});

async function removeBook() {
  if (typeof book.value !== 'number' || typeof title.value !== 'string') {
    return;
  }

  try {
    loading.value = true;
    await invoke(Command.RemoveBook, { id: book.value });
  } catch (err) {
    handleError(err);
  } finally {
    book.value = null;
    title.value = null;
    visible.value = false;
    loading.value = false;
  }
}
</script>

<template>
  <p-dialog
    v-model:visible="visible"
    header="Remove book"
    modal
    :closable="false"
    dismissable-mask
    block-scroll
    content-class="max-w-md"
  >
    <span class="p-text-secondary mb-6 block">{{ title }} will be removed from the library.</span>
    <div class="flex justify-end gap-2">
      <p-button
        type="button"
        label="Cancel"
        severity="secondary"
        :disabled="loading"
        @click="visible = false"
      />
      <p-button type="button" label="Remove" :loading :disabled="loading" @click="removeBook" />
    </div>
  </p-dialog>
</template>
