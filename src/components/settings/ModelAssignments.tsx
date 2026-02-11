import { useState, useEffect } from "react";
import { RefreshCw } from "lucide-react";
import { getAvailableModels } from "../../api/pipeline";
import type { AppConfig } from "../../types";

interface ModelAssignmentsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

const modelFields: { key: keyof AppConfig["models"]; label: string; hint?: string }[] = [
  { key: "ideator", label: "Ideator", hint: "Generates creative concepts from your idea" },
  { key: "composer", label: "Composer", hint: "Expands concepts into detailed descriptions" },
  { key: "judge", label: "Judge", hint: "Ranks and scores competing concepts" },
  { key: "promptEngineer", label: "Prompt Engineer", hint: "Converts descriptions into SD prompts" },
  { key: "reviewer", label: "Reviewer", hint: "Reviews final prompts for quality" },
  { key: "tagger", label: "Tagger (Vision)", hint: "Auto-tags images — needs a vision model" },
  { key: "captioner", label: "Captioner (Vision)", hint: "Auto-captions images — needs a vision model" },
];

export function ModelAssignments({ config, onChange }: ModelAssignmentsProps) {
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);

  const fetchModels = async () => {
    setLoading(true);
    setFetchError(null);
    try {
      const models = await getAvailableModels();
      setAvailableModels(models);
    } catch (e) {
      setFetchError(e instanceof Error ? e.message : "Failed to fetch models from Ollama");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchModels();
  }, []);

  const updateModel = (key: keyof AppConfig["models"], value: string) => {
    onChange({
      ...config,
      models: { ...config.models, [key]: value },
    });
  };

  return (
    <section className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
          Model Assignments
        </h3>
        <button
          onClick={fetchModels}
          disabled={loading}
          className="flex items-center gap-1.5 text-xs text-blue-400 hover:text-blue-300 disabled:text-zinc-600"
        >
          <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          {loading ? "Loading..." : "Refresh Models"}
        </button>
      </div>

      {fetchError && (
        <div className="bg-red-400/10 border border-red-400/20 rounded px-3 py-2 text-xs text-red-400">
          {fetchError} — Save your Ollama endpoint first, then refresh.
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {modelFields.map(({ key, label, hint }) => (
          <label key={key} className="block">
            <span className="text-sm text-zinc-400">{label}</span>
            {hint && <span className="block text-[10px] text-zinc-600 mb-0.5">{hint}</span>}
            {availableModels.length > 0 ? (
              <select
                value={config.models[key]}
                onChange={(e) => updateModel(key, e.target.value)}
                className="mt-1 block w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none appearance-none"
                style={{ colorScheme: "dark" }}
              >
                {/* Keep current value as option even if not in list */}
                {!availableModels.includes(config.models[key]) && (
                  <option value={config.models[key]} className="bg-zinc-800 text-zinc-100">
                    {config.models[key]} (not installed)
                  </option>
                )}
                {availableModels.map((model) => (
                  <option key={model} value={model} className="bg-zinc-800 text-zinc-100">
                    {model}
                  </option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={config.models[key]}
                onChange={(e) => updateModel(key, e.target.value)}
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                placeholder="e.g. mistral:7b"
              />
            )}
          </label>
        ))}
      </div>
    </section>
  );
}
