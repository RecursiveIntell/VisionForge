import { FolderOpen, Save } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useConfig } from "../../hooks/useConfig";
import { useToast } from "../shared/Toast";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { ConnectionSettings } from "./ConnectionSettings";
import { ModelAssignments } from "./ModelAssignments";
import { PipelinePrompts } from "./PipelinePrompts";
import { QualityPresets } from "./QualityPresets";
import { HardwareSettings } from "./HardwareSettings";
import type { AppConfig } from "../../types";

export function SettingsPanel() {
  const { config, loading, error, saving, save, update } = useConfig();
  const { addToast } = useToast();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <LoadingSpinner size={32} />
      </div>
    );
  }

  if (!config) {
    return (
      <div className="p-6 text-red-400">
        Failed to load configuration{error ? `: ${error}` : ""}
      </div>
    );
  }

  const handleSave = async () => {
    const ok = await save(config);
    if (ok) {
      addToast("success", "Settings saved");
    } else {
      addToast("error", "Failed to save settings");
    }
  };

  return (
    <div className="p-6 max-w-3xl mx-auto space-y-8">
      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded p-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <StorageSection config={config} onChange={update as (c: typeof config) => void} />
      <ConnectionSettings config={config} onChange={update as (c: typeof config) => void} onSave={() => save(config)} />
      <ModelAssignments config={config} onChange={update as (c: typeof config) => void} />
      <PipelinePrompts config={config} onChange={update as (c: typeof config) => void} />
      <QualityPresets config={config} onChange={update as (c: typeof config) => void} />
      <HardwareSettings config={config} onChange={update as (c: typeof config) => void} />

      <div className="flex justify-end pt-4 border-t border-zinc-700">
        <button
          onClick={handleSave}
          disabled={saving}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-600 text-white text-sm rounded"
        >
          {saving ? <LoadingSpinner size={16} /> : <Save size={16} />}
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>
    </div>
  );
}

function StorageSection({
  config,
  onChange,
}: {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}) {
  const handleBrowse = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Image Save Location",
    });
    if (selected) {
      onChange({
        ...config,
        storage: { ...config.storage, imageDirectory: selected as string },
      });
    }
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Storage
      </h3>
      <div>
        <div className="flex items-center gap-2 mb-1">
          <FolderOpen size={14} className="text-zinc-400" />
          <span className="text-sm text-zinc-400">Image Save Location</span>
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            value={config.storage?.imageDirectory ?? ""}
            onChange={(e) =>
              onChange({
                ...config,
                storage: { ...config.storage, imageDirectory: e.target.value },
              })
            }
            className="flex-1 bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
            placeholder="~/.visionforge/images (default)"
          />
          <button
            onClick={handleBrowse}
            className="px-3 py-2 bg-zinc-600 hover:bg-zinc-500 text-zinc-200 text-sm rounded border border-zinc-500 transition-colors"
          >
            Browse
          </button>
        </div>
        <p className="mt-1 text-xs text-zinc-500">
          Leave empty to use the default location. Changes apply after saving.
        </p>
      </div>
    </section>
  );
}
