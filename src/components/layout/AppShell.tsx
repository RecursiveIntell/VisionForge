import { useState, useEffect, useCallback } from "react";
import { Sidebar, type Page } from "./Sidebar";
import { Header } from "./Header";
import { BatchStatusBar } from "./BatchStatusBar";
import { SettingsPanel } from "../settings/SettingsPanel";
import { PromptStudio } from "../prompt-studio/PromptStudio";
import { GalleryView } from "../gallery/GalleryView";
import { QueuePanel } from "../queue/QueuePanel";
import { SeedLibrary } from "../seeds/SeedLibrary";
import { ComparisonView } from "../comparison/ComparisonView";
import { useAiBatchQueue } from "../../hooks/useAiBatchQueue";
import { useToast } from "../shared/Toast";

const pageShortcuts: Record<string, Page> = {
  "1": "prompt-studio",
  "2": "gallery",
  "3": "queue",
  "4": "seeds",
  "5": "comparison",
  "6": "settings",
};

const pageComponents: Record<Page, () => React.ReactNode> = {
  "prompt-studio": () => <PromptStudio />,
  "gallery": () => <GalleryView />,
  "queue": () => <QueuePanel />,
  "seeds": () => <SeedLibrary />,
  "comparison": () => <ComparisonView />,
  "settings": () => <SettingsPanel />,
};

const allPages: Page[] = Object.keys(pageComponents) as Page[];

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${minutes}m ${secs}s`;
}

export function AppShell() {
  const [currentPage, setCurrentPage] = useState<Page>("prompt-studio");
  const [visitedPages, setVisitedPages] = useState<Set<Page>>(() => new Set(["prompt-studio"]));
  const batchState = useAiBatchQueue();
  const { addToast } = useToast();

  const navigate = useCallback((page: Page) => {
    setCurrentPage(page);
    setVisitedPages((prev) => {
      if (prev.has(page)) return prev;
      const next = new Set(prev);
      next.add(page);
      return next;
    });
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      // Ctrl+1-6 for page navigation
      if (e.ctrlKey && !e.shiftKey && !e.altKey) {
        const page = pageShortcuts[e.key];
        if (page) {
          e.preventDefault();
          navigate(page);
        }
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [navigate]);

  // Toast on batch completion
  useEffect(() => {
    if (batchState.lastCompletion) {
      const s = batchState.lastCompletion;
      const opLabel = s.op === "tag" ? "Tagging" : "Captioning";
      const totalTime = formatDuration(s.totalDurationMs);
      const avgTime = formatDuration(s.avgDurationMs);

      if (s.failed > 0) {
        addToast(
          "warning",
          `${opLabel} complete: ${s.succeeded} succeeded, ${s.failed} failed — ${totalTime} total, ${avgTime} avg/image`
        );
      } else {
        addToast(
          "success",
          `${opLabel} complete: ${s.succeeded}/${s.total} — ${totalTime} total, ${avgTime} avg/image`
        );
      }
    }
  }, [batchState.lastCompletion, addToast]);

  return (
    <div className="flex h-screen bg-zinc-900">
      <Sidebar currentPage={currentPage} onNavigate={navigate} />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header currentPage={currentPage} />
        <main className="flex-1 overflow-auto relative">
          {allPages.map((key) => {
            if (!visitedPages.has(key)) return null;
            return (
              <div
                key={key}
                className={`absolute inset-0 overflow-auto ${
                  currentPage === key ? "visible z-10" : "invisible z-0"
                }`}
                aria-hidden={currentPage !== key}
              >
                {pageComponents[key]()}
              </div>
            );
          })}
        </main>
        <BatchStatusBar
          batchState={batchState}
          onExpand={() => navigate("queue")}
        />
      </div>
    </div>
  );
}
