import type { AppConfig } from "../../types";

interface ModelAssignmentsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

const modelFields: { key: keyof AppConfig["models"]; label: string }[] = [
  { key: "ideator", label: "Ideator" },
  { key: "composer", label: "Composer" },
  { key: "judge", label: "Judge" },
  { key: "promptEngineer", label: "Prompt Engineer" },
  { key: "reviewer", label: "Reviewer" },
  { key: "tagger", label: "Tagger (Vision)" },
  { key: "captioner", label: "Captioner (Vision)" },
];

export function ModelAssignments({ config, onChange }: ModelAssignmentsProps) {
  const updateModel = (key: keyof AppConfig["models"], value: string) => {
    onChange({
      ...config,
      models: { ...config.models, [key]: value },
    });
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Model Assignments
      </h3>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {modelFields.map(({ key, label }) => (
          <label key={key} className="block">
            <span className="text-sm text-zinc-400">{label}</span>
            <input
              type="text"
              value={config.models[key]}
              onChange={(e) => updateModel(key, e.target.value)}
              className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
            />
          </label>
        ))}
      </div>
    </section>
  );
}
