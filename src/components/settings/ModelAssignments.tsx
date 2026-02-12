import { useState, useEffect, useCallback } from "react";
import { RefreshCw, Brain, Plus, X } from "lucide-react";
import { getAvailableModels, getThinkingModels } from "../../api/pipeline";
import type { AppConfig } from "../../types";

interface ModelAssignmentsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

const MODEL_FIELDS: {
  key: keyof AppConfig["models"];
  stageKey: string;
  label: string;
  hint?: string;
  thinkHint?: string;
}[] = [
  {
    key: "ideator",
    stageKey: "ideator",
    label: "Ideator",
    hint: "Generates creative concepts from your idea",
    thinkHint: "Thinking can make concepts more cautious — usually better OFF for creativity",
  },
  {
    key: "composer",
    stageKey: "composer",
    label: "Composer",
    hint: "Expands concepts into detailed descriptions",
    thinkHint: "Thinking can over-structure descriptions — usually better OFF",
  },
  {
    key: "judge",
    stageKey: "judge",
    label: "Judge",
    hint: "Ranks and scores competing concepts",
    thinkHint: "Thinking improves analytical ranking — consider turning ON",
  },
  {
    key: "promptEngineer",
    stageKey: "promptEngineer",
    label: "Prompt Engineer",
    hint: "Converts descriptions into SD prompts",
    thinkHint: "Thinking can contaminate structured prompt output — usually better OFF",
  },
  {
    key: "reviewer",
    stageKey: "reviewer",
    label: "Reviewer",
    hint: "Reviews final prompts for quality",
    thinkHint: "Thinking helps catch issues — consider turning ON",
  },
  {
    key: "tagger",
    stageKey: "tagger",
    label: "Tagger (Vision)",
    hint: "Auto-tags images — needs a vision model",
  },
  {
    key: "captioner",
    stageKey: "captioner",
    label: "Captioner (Vision)",
    hint: "Auto-captions images — needs a vision model",
  },
];

export function ModelAssignments({ config, onChange }: ModelAssignmentsProps) {
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [thinkingModels, setThinkingModels] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [showCustomThinking, setShowCustomThinking] = useState(false);
  const [newCustomModel, setNewCustomModel] = useState("");

  const fetchModels = useCallback(async () => {
    setLoading(true);
    setFetchError(null);
    try {
      const [models, thinking] = await Promise.all([
        getAvailableModels(),
        getThinkingModels(),
      ]);
      setAvailableModels(models);
      setThinkingModels(new Set(thinking));
    } catch (e) {
      setFetchError(
        e instanceof Error ? e.message : "Failed to fetch models from Ollama"
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  const updateModel = (key: keyof AppConfig["models"], value: string) => {
    if (typeof value !== "string") return;
    onChange({
      ...config,
      models: { ...config.models, [key]: value },
    });
  };

  const getThinkingEnabled = (stageKey: string): boolean | undefined => {
    return config.models.thinkingOverrides?.[stageKey];
  };

  const setThinkingEnabled = (stageKey: string, enabled: boolean | undefined) => {
    const overrides = { ...(config.models.thinkingOverrides ?? {}) };
    if (enabled === undefined) {
      delete overrides[stageKey];
    } else {
      overrides[stageKey] = enabled;
    }
    onChange({
      ...config,
      models: { ...config.models, thinkingOverrides: overrides },
    });
  };

  const isThinkingModel = (modelName: string): boolean => {
    return thinkingModels.has(modelName);
  };

  const addCustomThinkingModel = () => {
    const trimmed = newCustomModel.trim();
    if (!trimmed) return;
    const existing = config.models.customThinkingModels ?? [];
    if (existing.includes(trimmed)) return;
    onChange({
      ...config,
      models: {
        ...config.models,
        customThinkingModels: [...existing, trimmed],
      },
    });
    setNewCustomModel("");
    setThinkingModels((prev) => new Set([...prev, trimmed]));
  };

  const removeCustomThinkingModel = (model: string) => {
    const existing = config.models.customThinkingModels ?? [];
    onChange({
      ...config,
      models: {
        ...config.models,
        customThinkingModels: existing.filter((m) => m !== model),
      },
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
          {loading ? "Detecting..." : "Refresh Models"}
        </button>
      </div>

      {fetchError && (
        <div className="bg-red-400/10 border border-red-400/20 rounded px-3 py-2 text-xs text-red-400">
          {fetchError} — Save your Ollama endpoint first, then refresh.
        </div>
      )}

      {thinkingModels.size > 0 && (
        <div className="bg-purple-400/5 border border-purple-400/15 rounded px-3 py-2 text-xs text-purple-300/80">
          <Brain size={11} className="inline mr-1 -mt-0.5" />
          {thinkingModels.size} thinking model{thinkingModels.size !== 1 ? "s" : ""} detected.
          Use the toggle to control thinking per stage.
        </div>
      )}

      <div className="space-y-3">
        {MODEL_FIELDS.map(({ key, stageKey, label, hint, thinkHint }) => {
          const currentModel = config.models[key];
          const modelCanThink =
            typeof currentModel === "string" && isThinkingModel(currentModel);
          const thinkingState = getThinkingEnabled(stageKey);

          return (
            <div key={key} className="block">
              <div className="flex items-center justify-between mb-0.5">
                <div>
                  <span className="text-sm text-zinc-400">{label}</span>
                  {hint && (
                    <span className="block text-[10px] text-zinc-600">
                      {hint}
                    </span>
                  )}
                </div>

                {modelCanThink && thinkHint && (
                  <button
                    onClick={() => {
                      if (thinkingState === undefined) {
                        setThinkingEnabled(stageKey, true);
                      } else if (thinkingState === true) {
                        setThinkingEnabled(stageKey, false);
                      } else {
                        setThinkingEnabled(stageKey, undefined);
                      }
                    }}
                    className={`flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium transition-colors ${
                      thinkingState === true
                        ? "bg-purple-500/20 text-purple-300 border border-purple-500/30"
                        : thinkingState === false
                        ? "bg-zinc-700/50 text-zinc-500 border border-zinc-600/30"
                        : "bg-zinc-800/50 text-zinc-500 border border-zinc-700/30"
                    }`}
                    title={
                      thinkHint +
                      "\n\nCurrent: " +
                      (thinkingState === true
                        ? "ON — model will reason before responding"
                        : thinkingState === false
                        ? "OFF — thinking explicitly disabled"
                        : "DEFAULT — think param omitted (model uses its default)")
                    }
                  >
                    <Brain size={10} />
                    {thinkingState === true
                      ? "Think ON"
                      : thinkingState === false
                      ? "Think OFF"
                      : "Think: Default"}
                  </button>
                )}
              </div>

              {availableModels.length > 0 ? (
                <select
                  value={typeof currentModel === "string" ? currentModel : ""}
                  onChange={(e) => updateModel(key, e.target.value)}
                  className="mt-1 block w-full bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none appearance-none"
                  style={{ colorScheme: "dark" }}
                >
                  {typeof currentModel === "string" &&
                    !availableModels.includes(currentModel) && (
                      <option
                        value={currentModel}
                        className="bg-zinc-800 text-zinc-100"
                      >
                        {currentModel} (not installed)
                      </option>
                    )}
                  {availableModels.map((model) => (
                    <option
                      key={model}
                      value={model}
                      className="bg-zinc-800 text-zinc-100"
                    >
                      {model}
                      {isThinkingModel(model) ? " \u{1F9E0}" : ""}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  type="text"
                  value={typeof currentModel === "string" ? currentModel : ""}
                  onChange={(e) => updateModel(key, e.target.value)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                  placeholder="e.g. mistral:7b"
                />
              )}
            </div>
          );
        })}
      </div>

      {/* Custom Thinking Models */}
      <div className="pt-2 border-t border-zinc-800">
        <button
          onClick={() => setShowCustomThinking(!showCustomThinking)}
          className="text-xs text-zinc-500 hover:text-zinc-400 flex items-center gap-1"
        >
          <Brain size={11} />
          {showCustomThinking ? "Hide" : "Manage"} custom thinking models
          {(config.models.customThinkingModels?.length ?? 0) > 0 && (
            <span className="text-zinc-600">
              ({config.models.customThinkingModels?.length})
            </span>
          )}
        </button>

        {showCustomThinking && (
          <div className="mt-2 space-y-2">
            <p className="text-[10px] text-zinc-600">
              Add model names that should be treated as thinking-capable.
              Most thinking models are auto-detected — only add custom or
              fine-tuned models here.
            </p>

            {(config.models.customThinkingModels ?? []).map((model) => (
              <div
                key={model}
                className="flex items-center justify-between bg-zinc-800 rounded px-2 py-1"
              >
                <span className="text-xs text-zinc-300">{model}</span>
                <button
                  onClick={() => removeCustomThinkingModel(model)}
                  className="text-zinc-600 hover:text-red-400"
                >
                  <X size={12} />
                </button>
              </div>
            ))}

            <div className="flex gap-1.5">
              <input
                type="text"
                value={newCustomModel}
                onChange={(e) => setNewCustomModel(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") addCustomThinkingModel();
                }}
                placeholder="e.g. my-custom-reasoner:7b"
                className="flex-1 bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-xs text-zinc-100 focus:border-blue-500 focus:outline-none"
              />
              <button
                onClick={addCustomThinkingModel}
                disabled={!newCustomModel.trim()}
                className="flex items-center gap-1 px-2 py-1 bg-zinc-700 border border-zinc-600 rounded text-xs text-zinc-300 hover:border-purple-500/50 disabled:opacity-30"
              >
                <Plus size={10} />
                Add
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
