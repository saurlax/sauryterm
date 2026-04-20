<script setup lang="ts">
const route = useRoute();
const router = useRouter();
const { tabs, activeTab, openTab, openSettingsTab, closeTab } = useTabs();

function isTerminalTabActive(tab: Tab) {
  return route.path === "/terminal" && route.query.ptyId === tab.id;
}

function isPageTabActive(tab: Tab) {
  return route.fullPath === tab.to;
}

const navigationItems = computed(() => [
  [
    ...tabs.value.map((tab) => ({
      label: tab.title,
      value: tab.id,
      active:
        tab.type === "terminal"
          ? isTerminalTabActive(tab)
          : isPageTabActive(tab),
      onSelect: (event: Event) => {
        event.preventDefault();
        if (route.fullPath === tab.to) {
          activeTab.value = tab.id;
          return;
        }
        activeTab.value = tab.id;
        void router.push(tab.to);
      },
    })),
    {
      icon: "i-lucide-plus",
      onSelect: (event: Event) => {
        event.preventDefault();
        event.stopPropagation();
        void onAddTab();
      },
    },
  ],
  [
    {
      icon: "i-lucide-settings",
      onSelect: (event: Event) => {
        event.preventDefault();
        event.stopPropagation();
        onOpenSettings();
      },
    },
  ],
]);

watch(
  () => route.fullPath,
  (fullPath) => {
    activeTab.value = tabs.value.find((item) => item.to === fullPath)?.id;
  },
  { immediate: true },
);

watch(activeTab, (tabId) => {
  if (!tabId) {
    return;
  }

  const tab = tabs.value.find((item) => item.id === tabId);
  if (!tab || route.fullPath === tab.to) {
    return;
  }

  void router.push(tab.to);
});

async function onAddTab() {
  await openTab({ command: "cmd.exe", title: "cmd" });
}

function onOpenSettings() {
  const tab = openSettingsTab();
  void router.push(tab.to);
}

async function onCloseTab(tabId: string | undefined) {
  if (!tabId) {
    return;
  }
  await closeTab(tabId);
}

function getItemValue(item: unknown) {
  if (!item || typeof item !== "object" || !("value" in item)) {
    return undefined;
  }

  const value = (item as { value?: unknown }).value;
  return typeof value === "string" ? value : undefined;
}

function isClosable(value: string | undefined) {
  if (!value) {
    return false;
  }
  return tabs.value.some((tab) => tab.id === value);
}
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden">
    <UNavigationMenu :items="navigationItems" variant="link" highlight>
      <template #item-trailing="{ item }">
        <UButton
          v-if="isClosable(getItemValue(item))"
          icon="i-lucide-x"
          color="neutral"
          variant="ghost"
          size="xs"
          type="button"
          aria-label="Close tab"
          @mousedown.stop.prevent
          @click.stop.prevent="onCloseTab(getItemValue(item))"
        />
      </template>
    </UNavigationMenu>
    <div class="min-h-0 flex-1">
      <slot />
    </div>
  </div>
</template>
