<script setup lang="ts">
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import "@xterm/xterm/css/xterm.css";
import type { Term } from "~/utils/term";

const props = defineProps<{
  term: Term | null;
}>();

const termRef = useTemplateRef("terminal");
const term = new Terminal({
  windowOptions: {},
});
const fitAddon = new FitAddon();
term.loadAddon(fitAddon);
term.loadAddon(new WebglAddon());
const current = shallowRef<Term | null>(null);
let disposeInputHandler: (() => void) | null = null;
let disposeTermData: (() => void) | null = null;
let disposeTermExit: (() => void) | null = null;

const onWindowResize = () => fitAddon.fit();

function detachTermListeners() {
  disposeTermData?.();
  disposeTermExit?.();
  disposeTermData = null;
  disposeTermExit = null;
}

function bindTerm(target: Term | null) {
  detachTermListeners();
  current.value = target;
  if (!target) {
    return;
  }

  disposeTermData = target.onData((data) => term.write(data));
  disposeTermExit = target.onExit((code) => {
    term.write(
      `\r\n\x1b[90m[shell exited${code === null ? "" : `: ${code}`}]\x1b[0m\r\n`,
    );
  });
}

onMounted(() => {
  term.open(termRef.value!);
  fitAddon.fit();
  window.addEventListener("resize", onWindowResize);

  const dataListener = term.onData((data) => {
    void current.value?.send(data);
  });
  disposeInputHandler = () => dataListener.dispose();
  bindTerm(props.term);
});

watch(
  () => props.term,
  (next) => {
    bindTerm(next);
  },
);

onBeforeUnmount(() => {
  window.removeEventListener("resize", onWindowResize);
  disposeInputHandler?.();
  disposeInputHandler = null;
  detachTermListeners();
  current.value = null;
  term.dispose();
});
</script>

<template>
  <div ref="terminal" class="h-full"></div>
</template>

<style>
.terminal,
.xterm-scrollable-element {
  height: 100%;
}
</style>
