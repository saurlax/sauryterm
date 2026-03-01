<script setup lang="ts">
const { activeTerm, openTab } = useTabs();
const opening = ref(false);

async function onOpenShell() {
  opening.value = true;
  try {
    await openTab();
  } finally {
    opening.value = false;
  }
}
</script>

<template>
  <Xterm v-if="activeTerm" :term="activeTerm" />
  <UEmpty v-else title="Select a terminal to get started." variant="naked">
    <template #actions>
      <UButton :loading="opening" @click="onOpenShell">Shell</UButton>
    </template>
  </UEmpty>
</template>
