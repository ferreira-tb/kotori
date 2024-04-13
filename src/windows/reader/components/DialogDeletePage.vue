<script setup lang="ts">
import { Command } from '@/utils/commands';

const visible = defineModel<boolean>({ required: true, default: false });

const props = defineProps<{
  page: number;
}>();

const loading = ref(false);

async function deletePage() {
  try {
    loading.value = true;
    await invoke(Command.DeleteBookPage, { page: props.page });
  } catch (err) {
    handleError(err);
  } finally {
    visible.value = false;
    loading.value = false;
  }
}
</script>

<template>
  <p-dialog
    v-model:visible="visible"
    header="Delete page"
    modal
    :closable="false"
    dismissable-mask
    block-scroll
  >
    <span class="p-text-secondary mb-6 block">Are you sure you want to delete this page?</span>
    <div class="flex justify-end gap-2">
      <p-button
        type="button"
        label="Cancel"
        severity="secondary"
        :disabled="loading"
        @click="visible = false"
      />
      <p-button type="button" label="Delete" :loading :disabled="loading" @click="deletePage" />
    </div>
  </p-dialog>
</template>
