import type { Page } from "./Sidebar";

interface HeaderProps {
  currentPage: Page;
}

const pageTitles: Record<Page, string> = {
  "prompt-studio": "Prompt Studio",
  gallery: "Gallery",
  queue: "Queue",
  seeds: "Seed Library",
  comparison: "A/B Comparisons",
  settings: "Settings",
};

export function Header({ currentPage }: HeaderProps) {
  return (
    <header className="h-12 bg-zinc-800 border-b border-zinc-700 flex items-center px-6">
      <h2 className="text-sm font-medium text-zinc-300">
        {pageTitles[currentPage]}
      </h2>
    </header>
  );
}
