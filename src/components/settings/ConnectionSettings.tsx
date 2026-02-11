import { useState } from "react";
import { checkComfyuiHealth } from "../../api/comfyui";
import { checkOllamaHealth } from "../../api/pipeline";
import type { AppConfig } from "../../types";

interface ConnectionSettingsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
  onSave?: () => void;
}

type HealthStatus = "idle" | "checking" | "ok" | "error";

export function ConnectionSettings({ config, onChange, onSave }: ConnectionSettingsProps) {
  const [comfyStatus, setComfyStatus] = useState<HealthStatus>("idle");
  const [ollamaStatus, setOllamaStatus] = useState<HealthStatus>("idle");

  const checkComfy = async () => {
    setComfyStatus("checking");
    try {
      onSave?.();
      // Small delay to let the save propagate to backend
      await new Promise((r) => setTimeout(r, 200));
      const ok = await checkComfyuiHealth();
      setComfyStatus(ok ? "ok" : "error");
    } catch {
      setComfyStatus("error");
    }
  };

  const checkOllama = async () => {
    setOllamaStatus("checking");
    try {
      onSave?.();
      await new Promise((r) => setTimeout(r, 200));
      const ok = await checkOllamaHealth();
      setOllamaStatus(ok ? "ok" : "error");
    } catch {
      setOllamaStatus("error");
    }
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Connections
      </h3>
      <div className="space-y-3">
        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-sm text-zinc-400">ComfyUI Endpoint</span>
            <div className="flex items-center gap-2">
              <StatusBadge status={comfyStatus} />
              <button
                onClick={checkComfy}
                disabled={comfyStatus === "checking"}
                className="text-xs text-blue-400 hover:text-blue-300 disabled:text-zinc-600"
              >
                {comfyStatus === "checking" ? "Checking..." : "Test"}
              </button>
            </div>
          </div>
          <input
            type="text"
            value={config.comfyui.endpoint}
            onChange={(e) => {
              setComfyStatus("idle");
              onChange({
                ...config,
                comfyui: { ...config.comfyui, endpoint: e.target.value },
              });
            }}
            className="block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
            placeholder="http://localhost:8188"
          />
        </div>
        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-sm text-zinc-400">Ollama Endpoint</span>
            <div className="flex items-center gap-2">
              <StatusBadge status={ollamaStatus} />
              <button
                onClick={checkOllama}
                disabled={ollamaStatus === "checking"}
                className="text-xs text-blue-400 hover:text-blue-300 disabled:text-zinc-600"
              >
                {ollamaStatus === "checking" ? "Checking..." : "Test"}
              </button>
            </div>
          </div>
          <input
            type="text"
            value={config.ollama.endpoint}
            onChange={(e) => {
              setOllamaStatus("idle");
              onChange({
                ...config,
                ollama: { ...config.ollama, endpoint: e.target.value },
              });
            }}
            className="block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
            placeholder="http://localhost:11434"
          />
        </div>
      </div>
    </section>
  );
}

function StatusBadge({ status }: { status: HealthStatus }) {
  if (status === "idle") return null;
  if (status === "checking") {
    return <span className="text-xs text-zinc-500">...</span>;
  }
  if (status === "ok") {
    return <span className="text-xs text-green-400">Connected</span>;
  }
  return <span className="text-xs text-red-400">Unreachable</span>;
}
