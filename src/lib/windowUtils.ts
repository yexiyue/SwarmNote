import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import {
  getAllWebviewWindows,
  getCurrentWebviewWindow,
  WebviewWindow,
} from "@tauri-apps/api/webviewWindow";

import { isMac } from "@/lib/utils";

const PICKER_WINDOW_WIDTH = 960;
const PICKER_WINDOW_HEIGHT = 580;
const APP_WINDOW_WIDTH = 1440;
const APP_WINDOW_HEIGHT = 900;

/**
 * Open a new Workspace Picker window.
 * If one already exists, focus it instead of creating a duplicate.
 */
export async function openPickerWindow() {
  const allWindows = await getAllWebviewWindows();
  const existing = allWindows.find((w) => w.label.startsWith("ws-"));
  if (existing) {
    await existing.setFocus();
    return;
  }

  const label = `ws-${Date.now()}`;

  new WebviewWindow(label, {
    url: "/?mode=picker",
    title: "SwarmNote",
    width: PICKER_WINDOW_WIDTH,
    height: PICKER_WINDOW_HEIGHT,
    minWidth: PICKER_WINDOW_WIDTH,
    minHeight: PICKER_WINDOW_HEIGHT,
    center: true,
    resizable: false,
    decorations: false,
    ...(isMac && {
      titleBarStyle: "overlay" as const,
      hiddenTitle: true,
      trafficLightPosition: new LogicalPosition(15, 16),
    }),
  });
}

/** Check if the current window was opened in picker mode. */
export function isPickerMode(): boolean {
  return new URLSearchParams(window.location.search).has("mode", "picker");
}

/**
 * Transition the current picker window into a full workspace app window.
 * Removes the picker query param, resizes, and updates the title.
 */
export async function transitionPickerToApp(workspaceName: string) {
  // Remove ?mode=picker from URL so re-renders see normal mode
  window.history.replaceState({}, "", "/");

  const win = getCurrentWebviewWindow();
  await win.setTitle(`${workspaceName} - SwarmNote`);
  await win.setResizable(true);
  await win.setMinSize(new LogicalSize(800, 600));
  await win.setSize(new LogicalSize(APP_WINDOW_WIDTH, APP_WINDOW_HEIGHT));
  await win.center();
}
