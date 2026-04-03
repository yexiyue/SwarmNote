import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { useEditorStore } from "@/stores/editorStore";

export type LoadState = "loading" | "loaded" | "retrying" | "failed";

const MAX_RETRIES = 3;
const RETRY_DELAYS = [1000, 2000, 4000]; // exponential backoff

interface UseMediaLoaderResult {
  state: LoadState;
  /** The resolved src with cache-busting param, available when state is "loaded" */
  src: string | undefined;
  /** Number of retries attempted so far */
  retryCount: number;
  /** Manually trigger a retry (resets retry count) */
  retry: () => void;
}

/**
 * Hook that probes whether a media file (image/video) is loadable via a hidden
 * Image element. Implements exponential backoff retry and listens for backend
 * `yjs:assets-updated` events to trigger immediate reload.
 *
 * @param resolvedUrl - The asset:// URL from resolveFileUrl
 * @param fileName - The filename to match against backend events (e.g. "screenshot-af3b.png")
 */
export function useMediaLoader(
  resolvedUrl: string | undefined,
  fileName: string,
): UseMediaLoaderResult {
  const [state, setState] = useState<LoadState>("loading");
  const [src, setSrc] = useState<string | undefined>();
  const [retryCount, setRetryCount] = useState(0);

  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);
  const retryCountRef = useRef(0);

  const probe = useCallback((url: string) => {
    if (!mountedRef.current) return;

    const img = new Image();
    img.onload = () => {
      if (!mountedRef.current) return;
      setState("loaded");
      // Cache-bust to ensure webview doesn't serve stale 404
      setSrc(`${url}${url.includes("?") ? "&" : "?"}t=${Date.now()}`);
    };
    img.onerror = () => {
      if (!mountedRef.current) return;

      const count = retryCountRef.current;
      if (count < MAX_RETRIES) {
        setState("retrying");
        const delay = RETRY_DELAYS[count] ?? RETRY_DELAYS[RETRY_DELAYS.length - 1];
        timerRef.current = setTimeout(() => {
          if (!mountedRef.current) return;
          retryCountRef.current = count + 1;
          setRetryCount(count + 1);
          probe(url);
        }, delay);
      } else {
        setState("failed");
      }
    };
    img.src = `${url}${url.includes("?") ? "&" : "?"}t=${Date.now()}`;
  }, []);

  // Start probing when resolvedUrl is available
  useEffect(() => {
    if (!resolvedUrl) return;

    mountedRef.current = true;
    retryCountRef.current = 0;
    setRetryCount(0);
    setState("loading");
    setSrc(undefined);
    probe(resolvedUrl);

    return () => {
      mountedRef.current = false;
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [resolvedUrl, probe]);

  // Listen for backend yjs:assets-updated event
  useEffect(() => {
    if (!resolvedUrl) return;

    const docUuid = useEditorStore.getState().docUuid;
    if (!docUuid) return;

    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    listen<{ docUuid: string; assets: string[] }>("yjs:assets-updated", (event) => {
      if (cancelled) return;
      if (event.payload.docUuid !== docUuid) return;
      if (!event.payload.assets.includes(fileName)) return;

      // Cancel any pending retry timer and probe immediately
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      retryCountRef.current = 0;
      setRetryCount(0);
      setState("loading");
      probe(resolvedUrl);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [resolvedUrl, fileName, probe]);

  const retry = useCallback(() => {
    if (!resolvedUrl) return;
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    retryCountRef.current = 0;
    setRetryCount(0);
    setState("loading");
    probe(resolvedUrl);
  }, [resolvedUrl, probe]);

  return { state, src, retryCount, retry };
}
