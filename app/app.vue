<script setup lang="ts">
const colorMode = useColorMode();
colorMode.preference = "dark";
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>

<style>
@import "tailwindcss";
@import "@nuxt/ui";

@theme {
  --color-primary: #11a8cd;
}
</style>
