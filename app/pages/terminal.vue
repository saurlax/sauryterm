<script setup lang="ts">
import { WTerm } from "@wterm/dom";

const route = useRoute();
const tabs = useState<Tab[]>("tabs", () => []);
const terminalRef = useTemplateRef<HTMLElement>("terminal");
const terminal = shallowRef<WTerm | null>(null);
const currentPty = shallowRef<PtyHandle | null>(null);

const ptyId = computed(() => {
  const value = route.query.ptyId;
  return typeof value === "string" ? value : null;
});

const pty = computed(() => {
  if (!ptyId.value) {
    return null;
  }

  return (
    tabs.value.find((tab) => tab.type === "terminal" && tab.id === ptyId.value)
      ?.pty ?? null
  );
});

let disposePtyData: (() => void) | null = null;
let disposePtyExit: (() => void) | null = null;
let bindVersion = 0;

function detachPtyListeners() {
  disposePtyData?.();
  disposePtyExit?.();
  disposePtyData = null;
  disposePtyExit = null;
}

function destroyTerminal() {
  detachPtyListeners();
  currentPty.value = null;
  terminal.value?.destroy();
  terminal.value = null;
}

async function bindPty(target: PtyHandle | null) {
  const version = ++bindVersion;
  destroyTerminal();

  if (!terminalRef.value) {
    return;
  }

  const instance = new WTerm(terminalRef.value, {
    autoResize: true,
    onData: (data) => {
      void currentPty.value?.send(data);
    },
  });

  await instance.init();

  if (version !== bindVersion) {
    instance.destroy();
    return;
  }

  terminal.value = instance;
  currentPty.value = target;

  if (!target) {
    return;
  }

  const context = target.getContext();
  if (version !== bindVersion) {
    return;
  }

  if (context) {
    instance.write(context);
  }

  disposePtyData = target.onData((data) => instance.write(data));
  disposePtyExit = target.onExit((code) => {
    instance.write(
      `\r\n\x1b[90m[shell exited${code === null ? "" : `: ${code}`} ]\x1b[0m\r\n`,
    );
  });
}

onMounted(async () => {
  await bindPty(pty.value);
});

watch(
  pty,
  (next) => {
    bindPty(next);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  bindVersion += 1;
  destroyTerminal();
});
</script>

<template>
  <div ref="terminal" class="h-full w-full" />
</template>

<style>
@import "@wterm/dom/css";

.wterm {
  width: 100%;
  height: 100%;
  border: none;
  border-radius: 0;
}
</style>
