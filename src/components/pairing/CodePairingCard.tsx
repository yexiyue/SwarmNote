import { Copy, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { PairingCodeInfo } from "@/commands/pairing";
import { generatePairingCode, getDeviceByCode, requestPairing } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type CardMode = "idle" | "generate" | "input";

export function CodePairingCard() {
  const [mode, setMode] = useState<CardMode>("idle");
  const [codeInfo, setCodeInfo] = useState<PairingCodeInfo | null>(null);
  const [remaining, setRemaining] = useState(0);
  const [inputCode, setInputCode] = useState("");
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const clearTimer = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const resetToIdle = useCallback(() => {
    clearTimer();
    setMode("idle");
    setCodeInfo(null);
    setRemaining(0);
    setInputCode("");
    setError(null);
  }, [clearTimer]);

  // Countdown timer
  useEffect(() => {
    if (mode !== "generate" || !codeInfo) return;

    const updateRemaining = () => {
      const left = Math.max(0, Math.floor((codeInfo.expiresAt - Date.now()) / 1000));
      setRemaining(left);
      if (left <= 0) {
        resetToIdle();
      }
    };

    updateRemaining();
    intervalRef.current = setInterval(updateRemaining, 1000);

    return clearTimer;
  }, [mode, codeInfo, clearTimer, resetToIdle]);

  async function handleGenerate() {
    setError(null);
    try {
      const info = await generatePairingCode(300);
      setCodeInfo(info);
      setMode("generate");
    } catch (e) {
      console.error("Failed to generate pairing code:", e);
      setError("生成配对码失败");
    }
  }

  async function handleRefresh() {
    clearTimer();
    await handleGenerate();
  }

  async function handleInputConnect() {
    if (inputCode.length !== 6) {
      setError("请输入6位配对码");
      return;
    }

    setConnecting(true);
    setError(null);
    try {
      const deviceInfo = await getDeviceByCode(inputCode);
      const resp = await requestPairing(deviceInfo.peerId, {
        type: "Code",
        code: inputCode,
      });
      if (resp.status === "Success") {
        resetToIdle();
      } else {
        setError(resp.reason ?? "配对被拒绝");
      }
    } catch (e) {
      console.error("Failed to connect with code:", e);
      setError("连接失败，请检查配对码");
    } finally {
      setConnecting(false);
    }
  }

  function handleCopyCode() {
    if (codeInfo) {
      navigator.clipboard.writeText(codeInfo.code).catch(console.error);
    }
  }

  function formatSeconds(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  if (mode === "idle") {
    return (
      <div className="rounded-lg border p-4">
        <div className="mb-2 text-sm font-medium">跨网络配对</div>
        <p className="mb-4 text-xs text-muted-foreground">在不同网络环境下，使用配对码连接设备</p>
        {error ? <p className="mb-3 text-xs text-destructive">{error}</p> : null}
        <div className="flex items-center gap-3">
          <Button size="sm" onClick={handleGenerate}>
            生成配对码
          </Button>
          <button
            type="button"
            className="text-xs text-muted-foreground underline hover:text-foreground"
            onClick={() => setMode("input")}
          >
            或 输入配对码连接
          </button>
        </div>
      </div>
    );
  }

  if (mode === "input") {
    return (
      <div className="rounded-lg border p-4">
        <div className="mb-2 text-sm font-medium">输入配对码</div>
        <p className="mb-4 text-xs text-muted-foreground">输入对方设备生成的6位配对码</p>
        {error ? <p className="mb-3 text-xs text-destructive">{error}</p> : null}
        <div className="flex items-center gap-2">
          <Input
            value={inputCode}
            onChange={(e) => setInputCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
            placeholder="000000"
            className="w-32 text-center font-mono tracking-widest"
            maxLength={6}
          />
          <Button size="sm" onClick={handleInputConnect} disabled={connecting}>
            {connecting ? "连接中..." : "连接"}
          </Button>
          <button
            type="button"
            className="text-xs text-muted-foreground underline hover:text-foreground"
            onClick={resetToIdle}
          >
            取消
          </button>
        </div>
      </div>
    );
  }

  // mode === "generate"
  return (
    <div className="rounded-lg border p-4">
      <div className="mb-2 text-sm font-medium">跨网络配对</div>
      <p className="mb-4 text-xs text-muted-foreground">
        将此配对码告知对方设备，配对码将在 {formatSeconds(remaining)} 后过期
      </p>

      <div className="mb-4 flex items-center gap-3">
        <div className="flex items-center gap-1 rounded-lg bg-muted px-4 py-2 font-mono text-2xl tracking-[0.3em]">
          {codeInfo?.code.split("").map((digit, _i, arr) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: fixed-length code digits, order never changes
            <span key={`${arr.length}-${_i}`}>{digit}</span>
          ))}
        </div>
        <Button variant="ghost" size="icon-sm" onClick={handleCopyCode} title="复制配对码">
          <Copy className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon-sm" onClick={handleRefresh} title="刷新码">
          <RefreshCw className="h-4 w-4" />
        </Button>
      </div>

      <div className="flex items-center gap-3">
        <span className="text-xs text-muted-foreground">等待对方连接...</span>
        <button
          type="button"
          className="text-xs text-muted-foreground underline hover:text-foreground"
          onClick={resetToIdle}
        >
          取消
        </button>
      </div>
    </div>
  );
}
