import { Awareness } from "y-protocols/awareness";
import type * as Y from "yjs";
import { applyYDocUpdate, closeYDoc } from "@/commands/document";

/**
 * Custom yjs provider that bridges BlockNote's collaboration layer with the
 * Tauri Rust backend via IPC. Uses the stable database UUID (not relPath)
 * to identify documents, so renames don't break the connection.
 */
export class TauriYjsProvider {
  public awareness: Awareness;
  public doc: Y.Doc;
  private _docUuid: string;
  private _destroying = false;

  constructor(doc: Y.Doc, docUuid: string) {
    this.doc = doc;
    this._docUuid = docUuid;
    this.awareness = new Awareness(doc);

    doc.on("update", this._onDocUpdate);
  }

  private _onDocUpdate = (update: Uint8Array, origin: unknown) => {
    if (this._destroying) return;
    if (origin === "remote") return;

    applyYDocUpdate(this._docUuid, Array.from(update)).catch((err) => {
      console.error("Failed to send yjs update to backend:", err);
    });
  };

  destroy() {
    this._destroying = true;
    this.doc.off("update", this._onDocUpdate);
    this.awareness.destroy();

    closeYDoc(this._docUuid).catch((err) => {
      console.error("Failed to close ydoc:", err);
    });
  }
}
