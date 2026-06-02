import type { ReactNode } from "react";
import { JA } from "../i18n/ja";

interface LayoutProps {
  pageTitle: string;
  onNavigateHome: () => void;
  onNavigateNew: () => void;
  children: ReactNode;
}

export function Layout({
  pageTitle,
  onNavigateHome,
  onNavigateNew,
  children,
}: LayoutProps) {
  return (
    <div className="flex h-screen bg-gray-50 text-gray-900">
      {/* サイドバー */}
      <aside
        className="w-56 flex-shrink-0 bg-primary-700 text-white flex flex-col"
        aria-label="サイドナビゲーション"
      >
        <div className="px-5 py-6 border-b border-primary-600">
          <span className="text-xl font-bold tracking-tight">{JA.appName}</span>
        </div>
        <nav className="flex-1 px-3 py-4 flex flex-col gap-1" aria-label="メインナビゲーション">
          <button
            type="button"
            onClick={onNavigateHome}
            aria-label={JA.nav.meetingList}
            className="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium hover:bg-primary-600 transition-colors text-left w-full"
          >
            <i className="fa-solid fa-folder-open w-4 text-center" aria-hidden="true" />
            {JA.nav.meetingList}
          </button>
          <button
            type="button"
            onClick={onNavigateNew}
            aria-label={JA.nav.newRecording}
            className="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium hover:bg-primary-600 transition-colors text-left w-full"
          >
            <i className="fa-solid fa-microphone w-4 text-center" aria-hidden="true" />
            {JA.nav.newRecording}
          </button>
        </nav>
      </aside>

      {/* メインエリア */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* ヘッダー */}
        <header className="bg-white border-b border-gray-200 px-6 py-4 flex-shrink-0">
          <h1 className="text-lg font-semibold text-gray-800">{pageTitle}</h1>
        </header>

        {/* コンテンツ */}
        <main className="flex-1 overflow-y-auto p-6" role="main">
          {children}
        </main>
      </div>
    </div>
  );
}
