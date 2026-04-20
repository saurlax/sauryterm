import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type ProcessDataEvent = {
  ptyId: string;
  data: string;
};

type ProcessExitEvent = {
  ptyId: string;
  code: number | null;
};

export type PtyOptions = {
  command?: string;
  args?: string[];
  ptyId?: string;
};

type DataHandler = (data: string) => void;
type ExitHandler = (code: number | null) => void;

const MAX_CONTEXT_BYTES = 1024 * 1024;

export class Pty {
  readonly ptyId: string;
  readonly command?: string;
  readonly args?: string[];
  private unlistenData: UnlistenFn | null = null;
  private unlistenExit: UnlistenFn | null = null;
  private dataHandlers = new Set<DataHandler>();
  private exitHandlers = new Set<ExitHandler>();
  private opened = false;
  private context = "";

  constructor(options: PtyOptions = {}) {
    this.ptyId = options.ptyId ?? crypto.randomUUID();
    this.command = options.command;
    this.args = options.args;
  }

  async open(): Promise<void> {
    if (this.opened) {
      return;
    }

    this.unlistenData = await listen<ProcessDataEvent>("pty://data", (event) => {
      if (event.payload.ptyId !== this.ptyId) {
        return;
      }

      this.context += event.payload.data;
      if (this.context.length > MAX_CONTEXT_BYTES) {
        this.context = this.context.slice(this.context.length - MAX_CONTEXT_BYTES);
      }

      for (const handler of this.dataHandlers) {
        handler(event.payload.data);
      }
    });

    this.unlistenExit = await listen<ProcessExitEvent>("pty://exit", (event) => {
      if (event.payload.ptyId !== this.ptyId) {
        return;
      }
      this.opened = false;
      for (const handler of this.exitHandlers) {
        handler(event.payload.code ?? null);
      }
    });

    await invoke("open_pty", {
      ptyId: this.ptyId,
      command: this.command,
      args: this.args,
    });

    this.opened = true;
  }

  async send(data: string): Promise<void> {
    await invoke("write_pty", { ptyId: this.ptyId, data });
  }

  getContext(): string {
    return this.context;
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
    await invoke("close_pty", { ptyId: this.ptyId });
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
    this.context = "";
  }
}
