import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type TermDataEvent = {
  termId: string;
  data: string;
};

type TermExitEvent = {
  termId: string;
  code: number | null;
};

export type TermOptions = {
  command?: string;
  args?: string[];
  termId?: string;
};

type DataHandler = (data: string) => void;
type ExitHandler = (code: number | null) => void;

export class Term {
  readonly termId: string;
  readonly command?: string;
  readonly args?: string[];
  private unlistenData: UnlistenFn | null = null;
  private unlistenExit: UnlistenFn | null = null;
  private dataHandlers = new Set<DataHandler>();
  private exitHandlers = new Set<ExitHandler>();
  private opened = false;

  constructor(options: TermOptions = {}) {
    this.termId = options.termId ?? crypto.randomUUID();
    this.command = options.command;
    this.args = options.args;
  }

  async open(): Promise<void> {
    if (this.opened) {
      return;
    }

    this.unlistenData = await listen<TermDataEvent>("term://data", (event) => {
      if (event.payload.termId !== this.termId) {
        return;
      }
      for (const handler of this.dataHandlers) {
        handler(event.payload.data);
      }
    });

    this.unlistenExit = await listen<TermExitEvent>("term://exit", (event) => {
      if (event.payload.termId !== this.termId) {
        return;
      }
      this.opened = false;
      for (const handler of this.exitHandlers) {
        handler(event.payload.code ?? null);
      }
    });

    await invoke("open_term", {
      termId: this.termId,
      command: this.command,
      args: this.args,
    });

    this.opened = true;
  }

  async send(data: string): Promise<void> {
    await invoke("write_term", { termId: this.termId, data });
  }

  onData(handler: DataHandler): () => void {
    this.dataHandlers.add(handler);
    return () => {
      this.dataHandlers.delete(handler);
    };
  }

  onExit(handler: ExitHandler): () => void {
    this.exitHandlers.add(handler);
    return () => {
      this.exitHandlers.delete(handler);
    };
  }

  async close(): Promise<void> {
    await invoke("close_term", { termId: this.termId });
    await this.dispose();
  }

  async dispose(): Promise<void> {
    this.unlistenData?.();
    this.unlistenExit?.();
    this.unlistenData = null;
    this.unlistenExit = null;
    this.dataHandlers.clear();
    this.exitHandlers.clear();
    this.opened = false;
  }
}
