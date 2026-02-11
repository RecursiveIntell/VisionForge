import { Save } from "lucide-react";
import { useConfig } from "../../hooks/useConfig";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { ConnectionSettings } from "./ConnectionSettings";
import { ModelAssignments } from "./ModelAssignments";
import { PipelinePrompts } from "./PipelinePrompts";
import { QualityPresets } from "./QualityPresets";
import { HardwareSettings } from "./HardwareSettings";

export function SettingsPanel() {
  const { config, loading, error, saving, save, update } = useConfig();

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

  const handleSave = () => {
    save(config);
  };

  return (
    <div className="p-6 max-w-3xl mx-auto space-y-8">
      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded p-3 text-sm text-red-300">
          {error}
        </div>
      )}

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
