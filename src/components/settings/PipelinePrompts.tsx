import type { AppConfig } from "../../types";

interface PipelinePromptsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

const stageToggles: { key: keyof AppConfig["pipeline"]; label: string }[] = [
  { key: "enableIdeator", label: "Ideator" },
  { key: "enableComposer", label: "Composer" },
  { key: "enableJudge", label: "Judge" },
  { key: "enablePromptEngineer", label: "Prompt Engineer" },
  { key: "enableReviewer", label: "Reviewer" },
];

export function PipelinePrompts({ config, onChange }: PipelinePromptsProps) {
  const toggle = (key: keyof AppConfig["pipeline"]) => {
    onChange({
      ...config,
      pipeline: { ...config.pipeline, [key]: !config.pipeline[key] },
    });
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Pipeline Stages
      </h3>
      <div className="space-y-2">
        {stageToggles.map(({ key, label }) => (
          <label key={key} className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={config.pipeline[key] as boolean}
              onChange={() => toggle(key)}
              className="w-4 h-4 rounded bg-zinc-700 border-zinc-600 text-blue-500 focus:ring-blue-500 focus:ring-offset-zinc-800"
            />
            <span className="text-sm text-zinc-300">{label}</span>
          </label>
        ))}
        <div className="pt-2 border-t border-zinc-700">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={config.pipeline.autoApprove}
              onChange={() =>
                onChange({
                  ...config,
                  pipeline: {
                    ...config.pipeline,
                    autoApprove: !config.pipeline.autoApprove,
                  },
                })
              }
              className="w-4 h-4 rounded bg-zinc-700 border-zinc-600 text-blue-500 focus:ring-blue-500 focus:ring-offset-zinc-800"
            />
            <span className="text-sm text-zinc-300">Auto-approve (skip approval gate)</span>
          </label>
        </div>
      </div>
    </section>
  );
}
