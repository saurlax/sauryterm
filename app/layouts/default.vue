<script setup lang="ts">
const { tabs, activeTab } = useTabs();
</script>

<template>
  <UMain class="h-screen flex flex-col">
    <UTabs
      v-if="tabs?.length"
      v-model="activeTab"
      :items="tabs"
      variant="link"
      :content="false"
    />
    <slot />
  </UMain>
</template>
