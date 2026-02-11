import {
  Wand2,
  Images,
  ListOrdered,
  Sprout,
  GitCompareArrows,
  Settings,
} from "lucide-react";

export type Page = "prompt-studio" | "gallery" | "queue" | "seeds" | "comparison" | "settings";

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { page: Page; label: string; icon: React.ReactNode }[] = [
  { page: "prompt-studio", label: "Prompt Studio", icon: <Wand2 size={20} /> },
  { page: "gallery", label: "Gallery", icon: <Images size={20} /> },
  { page: "queue", label: "Queue", icon: <ListOrdered size={20} /> },
  { page: "seeds", label: "Seeds", icon: <Sprout size={20} /> },
  { page: "comparison", label: "Compare", icon: <GitCompareArrows size={20} /> },
  { page: "settings", label: "Settings", icon: <Settings size={20} /> },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-56 bg-zinc-800 border-r border-zinc-700 flex flex-col">
      <div className="p-4 border-b border-zinc-700">
        <h1 className="text-lg font-bold text-zinc-100">VisionForge</h1>
      </div>
      <nav className="flex-1 py-2">
        {navItems.map((item) => (
          <button
            key={item.page}
            onClick={() => onNavigate(item.page)}
            className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm text-left ${
              currentPage === item.page
                ? "bg-blue-600/20 text-blue-400 border-r-2 border-blue-500"
                : "text-zinc-400 hover:text-zinc-100 hover:bg-zinc-700/50"
            }`}
          >
            {item.icon}
            {item.label}
          </button>
        ))}
      </nav>
    </aside>
  );
}
