interface PtyHandle {
  ptyId: string;
  command?: string;
  args?: string[];
  open(): Promise<void>;
  send(data: string): Promise<void>;
  getContext(): string;
  onData(handler: (data: string) => void): () => void;
  onExit(handler: (code: number | null) => void): () => void;
  close(): Promise<void>;
  dispose(): Promise<void>;
}

interface Tab {
  id: string;
  title: string;
  type: "terminal" | "page";
  to: string;
  pty?: PtyHandle;
}

interface TerminalTab extends Tab {
  type: "terminal";
  pty: PtyHandle;
}
