import type { NavigationMenuItem } from "@nuxt/ui";
import { Pty as PtyClient } from "~/utils/pty";

type TabItems = NavigationMenuItem[];
type PtyOptions = ConstructorParameters<typeof PtyClient>[0];

export default function useTabs() {
  const tabs = useState<Tab[]>("tabs", () => []);
  const activeTab = useState<string | undefined>(
    "active-pty-tab",
    () => undefined,
  );

  const tabItems = computed<TabItems>(() =>
    tabs.value.map(({ id, title, to }) => ({
      label: title,
      value: id,
      to,
    })),
  );

  async function openTab(options: PtyOptions & { title?: string } = {}) {
    const pty = new PtyClient({
      command: options.command,
      args: options.args,
    });
    await pty.open();

    const tab: TerminalTab = {
      id: pty.ptyId,
      title: options.title ?? options.command ?? "shell",
      type: "terminal",
      to: `/terminal?ptyId=${pty.ptyId}`,
      pty,
    };

    tabs.value.push(tab);
    activeTab.value = pty.ptyId;
    return pty;
  }

  function openPageTab(to: string, title: string) {
    const tab: Tab = {
      id: crypto.randomUUID(),
      title,
      type: "page",
      to,
    };

    tabs.value.push(tab);
    activeTab.value = tab.id;
    return tab;
  }

  function openOrFocusPageTab(to: string, title: string) {
    const existing = tabs.value.find((item) => item.type === "page" && item.to === to);
    if (existing) {
      activeTab.value = existing.id;
      return existing;
    }

    return openPageTab(to, title);
  }

  function openSettingsTab() {
    return openOrFocusPageTab("/settings", "Settings");
  }

  async function closeTab(ptyId: string | undefined) {
    if (!ptyId) {
      return;
    }

    const tab = tabs.value.find((item) => item.id === ptyId);
    if (tab?.type === "terminal" && tab.pty) {
      await tab.pty.close();
    }

    const index = tabs.value.findIndex((item) => item.id === ptyId);
    if (index === -1) {
      return;
    }

    tabs.value.splice(index, 1);

    if (activeTab.value === ptyId) {
      const next = tabs.value[index] ?? tabs.value[index - 1];
      activeTab.value = next?.id;
    }
  }

  return {
    tabs,
    tabItems,
    activeTab,
    openTab,
    openPageTab,
    openOrFocusPageTab,
    openSettingsTab,
    closeTab,
  };
}
