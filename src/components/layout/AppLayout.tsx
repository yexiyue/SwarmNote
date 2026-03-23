import type { ReactNode } from "react";
import { Sidebar } from "@/components/layout/Sidebar";
import { TitleBar } from "@/components/layout/TitleBar";
import { isMac } from "@/lib/utils";

interface AppLayoutProps {
  children: ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  if (isMac) {
    // macOS Notion-style: Sidebar full-height left | TitleBar+Editor right column
    return (
      <div className="flex h-screen overflow-hidden bg-background">
        <Sidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <TitleBar />
          {children}
        </div>
      </div>
    );
  }

  // Windows/Linux: TitleBar full-width top + Sidebar|Editor below
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        {children}
      </div>
    </div>
  );
}
