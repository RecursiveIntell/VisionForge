import { useState, useEffect } from "react";
import { Sidebar, type Page } from "./Sidebar";
import { Header } from "./Header";
import { SettingsPanel } from "../settings/SettingsPanel";
import { PromptStudio } from "../prompt-studio/PromptStudio";
import { GalleryView } from "../gallery/GalleryView";
import { QueuePanel } from "../queue/QueuePanel";
import { SeedLibrary } from "../seeds/SeedLibrary";
import { ComparisonView } from "../comparison/ComparisonView";

const pageShortcuts: Record<string, Page> = {
  "1": "prompt-studio",
  "2": "gallery",
  "3": "queue",
  "4": "seeds",
  "5": "comparison",
  "6": "settings",
};

export function AppShell() {
  const [currentPage, setCurrentPage] = useState<Page>("prompt-studio");

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      // Ctrl+1-6 for page navigation
      if (e.ctrlKey && !e.shiftKey && !e.altKey) {
        const page = pageShortcuts[e.key];
        if (page) {
          e.preventDefault();
          setCurrentPage(page);
        }
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  return (
    <div className="flex h-screen bg-zinc-900">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header currentPage={currentPage} />
        <main className="flex-1 overflow-auto">
          <PageContent page={currentPage} />
        </main>
      </div>
    </div>
  );
}

function PageContent({ page }: { page: Page }) {
  switch (page) {
    case "settings":
      return <SettingsPanel />;
    case "prompt-studio":
      return <PromptStudio />;
    case "gallery":
      return <GalleryView />;
    case "queue":
      return <QueuePanel />;
    case "seeds":
      return <SeedLibrary />;
    case "comparison":
      return <ComparisonView />;
  }
}
