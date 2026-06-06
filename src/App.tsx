import { useEffect, useState, useCallback } from "react";
import { JA } from "./i18n/ja";
import { useTauri } from "./hooks/useTauri";
import type { GenerateResult } from "./types/index";
import { Layout } from "./components/Layout";
import { ModelError } from "./pages/ModelError";
import { MeetingList } from "./pages/MeetingList";
import { Recording } from "./pages/Recording";
import { Transcript } from "./pages/Transcript";
import { Result } from "./pages/Result";

type Page =
  | { name: "loading" }
  | { name: "model_error" }
  | { name: "meeting_list" }
  | { name: "recording"; meetingId: string }
  | { name: "transcript"; meetingId: string; rawText: string }
  | { name: "result"; meetingId: string };

function pageTitleOf(page: Page): string {
  switch (page.name) {
    case "loading":
      return JA.common.loading;
    case "model_error":
      return JA.modelError.title;
    case "meeting_list":
      return JA.meetingList.title;
    case "recording":
      return JA.recording.title;
    case "transcript":
      return JA.transcript.title;
    case "result":
      return JA.result.title;
  }
}

export function App() {
  const { checkModel, initApp, createMeeting, getMeeting } = useTauri();
  const [page, setPage] = useState<Page>({ name: "loading" });
  const [initError, setInitError] = useState<string | null>(null);

  const initialize = useCallback(async () => {
    setPage({ name: "loading" });
    setInitError(null);

    const modelResult = await checkModel();
    if (modelResult.error !== null) {
      setPage({ name: "model_error" });
      return;
    }
    const modelOk = modelResult.data ?? false;
    if (!modelOk) {
      setPage({ name: "model_error" });
      return;
    }

    const initResult = await initApp();
    if (initResult.error !== null) {
      setInitError(initResult.error);
      setPage({ name: "model_error" });
      return;
    }

    setPage({ name: "meeting_list" });
  }, [checkModel, initApp]);

  useEffect(() => {
    void initialize();
  }, [initialize]);

  const handleNewRecording = useCallback(async () => {
    const nowLabel = new Date().toLocaleString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
    const title = `会議 ${nowLabel}`;
    const result = await createMeeting(title);
    if (result.error !== null || result.data === null) {
      setInitError(result.error ?? "会議の作成に失敗しました");
      return;
    }
    setPage({ name: "recording", meetingId: result.data });
  }, [createMeeting]);

  const handleOpenMeeting = useCallback(async (meetingId: string) => {
    const result = await getMeeting(meetingId);
    const status = result.data?.meeting.status ?? "done";
    if (status === "recording") {
      setPage({ name: "recording", meetingId });
    } else if (status === "processing") {
      const rawText =
        result.data?.transcript?.edited_text ??
        result.data?.transcript?.raw_text ??
        "";
      setPage({ name: "transcript", meetingId, rawText });
    } else {
      setPage({ name: "result", meetingId });
    }
  }, [getMeeting]);

  const handleRecordingComplete = useCallback(
    (meetingId: string, rawText: string) => {
      setPage({ name: "transcript", meetingId, rawText });
    },
    []
  );

  const handleTranscriptComplete = useCallback(
    (meetingId: string, _result: GenerateResult) => {
      setPage({ name: "result", meetingId });
    },
    []
  );

  const navigateHome = useCallback(() => {
    setPage({ name: "meeting_list" });
  }, []);

  if (page.name === "loading") {
    return (
      <div
        className="flex items-center justify-center h-screen bg-gray-50"
        role="status"
        aria-live="polite"
      >
        <i className="fa-solid fa-spinner animate-spin text-3xl text-primary-500 mr-3" aria-hidden="true" />
        <span className="text-gray-600 text-lg">{JA.common.loading}</span>
      </div>
    );
  }

  if (page.name === "model_error") {
    return (
      <div className="h-screen bg-gray-50">
        <ModelError onRetry={() => void initialize()} />
        {initError !== null && (
          <p className="text-center text-xs text-red-500 pb-4">{initError}</p>
        )}
      </div>
    );
  }

  return (
    <Layout
      pageTitle={pageTitleOf(page)}
      onNavigateHome={navigateHome}
      onNavigateNew={() => void handleNewRecording()}
    >
      {page.name === "meeting_list" && (
        <MeetingList
          onNewRecording={() => void handleNewRecording()}
          onOpenMeeting={(id) => void handleOpenMeeting(id)}
        />
      )}

      {page.name === "recording" && (
        <Recording
          meetingId={page.meetingId}
          onComplete={(rawText) => handleRecordingComplete(page.meetingId, rawText)}
          onBack={navigateHome}
        />
      )}

      {page.name === "transcript" && (
        <Transcript
          meetingId={page.meetingId}
          rawText={page.rawText}
          onComplete={(result) => handleTranscriptComplete(page.meetingId, result)}
          onBack={navigateHome}
        />
      )}

      {page.name === "result" && (
        <Result
          meetingId={page.meetingId}
          onBack={navigateHome}
        />
      )}
    </Layout>
  );
}
