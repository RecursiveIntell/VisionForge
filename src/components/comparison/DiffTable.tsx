import type { ImageEntry } from "../../types";

interface DiffTableProps {
  imageA: ImageEntry;
  imageB: ImageEntry;
  variableChanged: string;
}

type DiffField = {
  label: string;
  valueA: string | undefined;
  valueB: string | undefined;
};

export function DiffTable({ imageA, imageB, variableChanged }: DiffTableProps) {
  const fields: DiffField[] = [
    { label: "Checkpoint", valueA: imageA.checkpoint, valueB: imageB.checkpoint },
    { label: "Seed", valueA: imageA.seed?.toString(), valueB: imageB.seed?.toString() },
    { label: "Steps", valueA: imageA.steps?.toString(), valueB: imageB.steps?.toString() },
    { label: "CFG", valueA: imageA.cfgScale?.toString(), valueB: imageB.cfgScale?.toString() },
    { label: "Sampler", valueA: imageA.sampler, valueB: imageB.sampler },
    { label: "Scheduler", valueA: imageA.scheduler, valueB: imageB.scheduler },
    {
      label: "Resolution",
      valueA: imageA.width && imageA.height ? `${imageA.width}x${imageA.height}` : undefined,
      valueB: imageB.width && imageB.height ? `${imageB.width}x${imageB.height}` : undefined,
    },
  ];

  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg overflow-hidden">
      <div className="px-3 py-2 bg-zinc-700/50 border-b border-zinc-700">
        <span className="text-xs text-zinc-400">Variable changed: </span>
        <span className="text-xs text-blue-400 font-medium">
          {variableChanged}
        </span>
      </div>
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b border-zinc-700">
            <th className="text-left px-3 py-1.5 text-zinc-500 font-normal">
              Field
            </th>
            <th className="text-left px-3 py-1.5 text-zinc-500 font-normal">
              Image A
            </th>
            <th className="text-left px-3 py-1.5 text-zinc-500 font-normal">
              Image B
            </th>
          </tr>
        </thead>
        <tbody>
          {fields.map((field) => {
            const isDiff = field.valueA !== field.valueB;
            return (
              <tr
                key={field.label}
                className={`border-b border-zinc-700/50 ${
                  isDiff ? "bg-blue-400/5" : ""
                }`}
              >
                <td className="px-3 py-1.5 text-zinc-400">{field.label}</td>
                <td
                  className={`px-3 py-1.5 ${
                    isDiff ? "text-amber-400" : "text-zinc-300"
                  }`}
                >
                  {field.valueA ?? "-"}
                </td>
                <td
                  className={`px-3 py-1.5 ${
                    isDiff ? "text-amber-400" : "text-zinc-300"
                  }`}
                >
                  {field.valueB ?? "-"}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
