import type { TabsProps } from "@nuxt/ui";
import { Term, type TermOptions } from "~/utils/term";

type TabItems = NonNullable<TabsProps["items"]>;
type TermMap = Record<string, Term>;

export default function useTabs() {
  const tabs = useState<TabItems>("tabs", () => []);
  const activeTab = useState<string | undefined>(
    "active-term-tab",
    () => undefined,
  );
  const terms = useState<TermMap>("term-map", () => ({}));

  const activeTerm = computed<Term | null>(() => {
    const termId = activeTab.value;
    if (!termId) {
      return null;
    }
    return terms.value[termId] ?? null;
  });

  async function openTab(options: TermOptions = {}) {
    const term = new Term(options);
    await term.open();

    tabs.value.push({
      label: `Shell ${tabs.value.length + 1}`,
      icon: "i-lucide-terminal",
      value: term.termId,
    });

    terms.value[term.termId] = term;
    activeTab.value = term.termId;
    return term;
  }

  async function closeTab(termId: string | undefined = activeTab.value) {
    if (!termId) {
      return;
    }

    const term = terms.value[termId];
    if (term) {
      await term.close();
      delete terms.value[termId];
    }

    const index = tabs.value.findIndex(
      (item) => (item as { value?: string }).value === termId,
    );
    if (index === -1) {
      return;
    }

    tabs.value.splice(index, 1);

    if (activeTab.value === termId) {
      const next = tabs.value[index] ?? tabs.value[index - 1];
      activeTab.value = (next as { value?: string } | undefined)?.value;
    }
  }

  return { tabs, activeTab, activeTerm, openTab, closeTab };
}
