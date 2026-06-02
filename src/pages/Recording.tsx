import { useEffect, useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { JA } from "../i18n/ja";
import { useTauri } from "../hooks/useTauri";

type RecordingTab = "mic" | "file";

interface RecordingProps {
  meetingId: string;
  onComplete: (rawText: string) => void;
  onBack: () => void;
}

export function Recording({ meetingId, onComplete, onBack }: RecordingProps) {
  const { listAudioDevices, startRecording, stopRecording, importAudio } = useTauri();
  const [activeTab, setActiveTab] = useState<RecordingTab>("mic");

  // マイク録音
  const [devices, setDevices] = useState<string[]>([]);
  const [deviceIndex, setDeviceIndex] = useState<number>(0);
  const [isLoadingDevices, setIsLoadingDevices] = useState(false);
  const [devicesError, setDevicesError] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [recordError, setRecordError] = useState<string | null>(null);

  // ファイルインポート
  const [isImporting, setIsImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  const fetchDevices = useCallback(async () => {
    setIsLoadingDevices(true);
    setDevicesError(null);
    const result = await listAudioDevices();
    if (result.error !== null) {
      setDevicesError(result.error);
    } else {
      setDevices(result.data ?? []);
      setDeviceIndex(0);
    }
    setIsLoadingDevices(false);
  }, [listAudioDevices]);

  useEffect(() => {
    if (activeTab === "mic") {
      void fetchDevices();
    }
  }, [activeTab, fetchDevices]);

  const handleStartRecording = useCallback(async () => {
    setRecordError(null);
    const result = await startRecording(deviceIndex, meetingId);
    if (result.error !== null) {
      setRecordError(result.error);
      return;
    }
    setIsRecording(true);
  }, [deviceIndex, meetingId, startRecording]);

  const handleStopRecording = useCallback(async () => {
    setRecordError(null);
    const result = await stopRecording(meetingId);
    if (result.error !== null) {
      setRecordError(result.error);
      setIsRecording(false);
      return;
    }
    setIsRecording(false);
    const rawText = result.data ?? "";
    onComplete(rawText);
  }, [meetingId, stopRecording, onComplete]);

  const handleFileSelect = useCallback(async () => {
    setImportError(null);
    let filePath: string | null = null;
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "音声ファイル",
            extensions: ["wav", "mp3", "m4a", "webm"],
          },
        ],
      });
      if (selected === null || Array.isArray(selected)) {
        return;
      }
      filePath = selected;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "ファイル選択に失敗しました";
      setImportError(message);
      return;
    }

    setIsImporting(true);
    const result = await importAudio(filePath, meetingId);
    if (result.error !== null) {
      setImportError(result.error);
      setIsImporting(false);
      return;
    }
    setIsImporting(false);
    const rawText = result.data ?? "";
    onComplete(rawText);
  }, [meetingId, importAudio, onComplete]);

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      {/* タブ */}
      <div className="flex border-b border-gray-200" role="tablist" aria-label="録音方法を選択">
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "mic"}
          aria-controls="tab-panel-mic"
          id="tab-mic"
          onClick={() => setActiveTab("mic")}
          className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "mic"
              ? "border-primary-600 text-primary-700"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          <i className="fa-solid fa-microphone" aria-hidden="true" />
          {JA.recording.tabMic}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "file"}
          aria-controls="tab-panel-file"
          id="tab-file"
          onClick={() => setActiveTab("file")}
          className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "file"
              ? "border-primary-600 text-primary-700"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          <i className="fa-solid fa-file-audio" aria-hidden="true" />
          {JA.recording.tabFile}
        </button>
      </div>

      {/* マイク録音タブ */}
      {activeTab === "mic" && (
        <div
          id="tab-panel-mic"
          role="tabpanel"
          aria-labelledby="tab-mic"
          className="flex flex-col gap-5"
        >
          {isLoadingDevices ? (
            <div className="flex items-center gap-2 text-gray-500 text-sm" role="status">
              <i className="fa-solid fa-spinner animate-spin" aria-hidden="true" />
              {JA.common.loading}
            </div>
          ) : devicesError !== null ? (
            <div className="flex flex-col gap-3" role="alert">
              <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded">{devicesError}</p>
              <button
                type="button"
                onClick={() => void fetchDevices()}
                className="self-start text-sm text-primary-600 hover:underline"
                aria-label="デバイス一覧を再取得"
              >
                <i className="fa-solid fa-rotate-right mr-1" aria-hidden="true" />
                再取得
              </button>
            </div>
          ) : (
            <div className="flex flex-col gap-2">
              <label
                htmlFor="device-select"
                className="text-sm font-medium text-gray-700"
              >
                {JA.recording.deviceLabel}
              </label>
              <select
                id="device-select"
                value={deviceIndex}
                onChange={(e) => setDeviceIndex(Number(e.target.value))}
                disabled={isRecording}
                className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm text-gray-800 focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
                aria-label={JA.recording.deviceLabel}
              >
                {devices.length === 0 ? (
                  <option value={0}>デバイスが見つかりません</option>
                ) : (
                  devices.map((device, idx) => (
                    <option key={idx} value={idx}>
                      {device}
                    </option>
                  ))
                )}
              </select>
            </div>
          )}

          {/* 録音中アニメーション */}
          {isRecording && (
            <div
              className="flex items-center gap-3 px-4 py-3 bg-red-50 border border-red-200 rounded-lg"
              role="status"
              aria-live="polite"
            >
              <span className="inline-block w-3 h-3 bg-red-500 rounded-full animate-pulse" aria-hidden="true" />
              <span className="text-sm font-medium text-red-700">{JA.recording.recording}</span>
            </div>
          )}

          {recordError !== null && (
            <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
              {recordError}
            </p>
          )}

          <div className="flex gap-3">
            {!isRecording ? (
              <button
                type="button"
                onClick={() => void handleStartRecording()}
                disabled={isLoadingDevices || devices.length === 0}
                aria-label={JA.recording.startButton}
                className="flex items-center gap-2 px-5 py-2.5 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <i className="fa-solid fa-microphone" aria-hidden="true" />
                {JA.recording.startButton}
              </button>
            ) : (
              <button
                type="button"
                onClick={() => void handleStopRecording()}
                aria-label={JA.recording.stopButton}
                className="flex items-center gap-2 px-5 py-2.5 text-sm font-semibold text-white bg-red-600 hover:bg-red-700 rounded-md transition-colors"
              >
                <i className="fa-solid fa-stop" aria-hidden="true" />
                {JA.recording.stopButton}
              </button>
            )}
            <button
              type="button"
              onClick={onBack}
              disabled={isRecording}
              aria-label={JA.common.back}
              className="px-4 py-2.5 text-sm font-medium text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
            >
              {JA.common.back}
            </button>
          </div>
        </div>
      )}

      {/* ファイルインポートタブ */}
      {activeTab === "file" && (
        <div
          id="tab-panel-file"
          role="tabpanel"
          aria-labelledby="tab-file"
          className="flex flex-col gap-5"
        >
          <p className="text-sm text-gray-500">{JA.recording.fileFormats}</p>

          {isImporting && (
            <div
              className="flex items-center gap-2 px-4 py-3 bg-yellow-50 border border-yellow-200 rounded-lg"
              role="status"
              aria-live="polite"
            >
              <i className="fa-solid fa-spinner animate-spin text-yellow-600" aria-hidden="true" />
              <span className="text-sm font-medium text-yellow-700">{JA.recording.importing}</span>
            </div>
          )}

          {importError !== null && (
            <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
              {importError}
            </p>
          )}

          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => void handleFileSelect()}
              disabled={isImporting}
              aria-label={JA.recording.fileSelectButton}
              className="flex items-center gap-2 px-5 py-2.5 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <i className="fa-solid fa-file-audio" aria-hidden="true" />
              {JA.recording.fileSelectButton}
            </button>
            <button
              type="button"
              onClick={onBack}
              disabled={isImporting}
              aria-label={JA.common.back}
              className="px-4 py-2.5 text-sm font-medium text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
            >
              {JA.common.back}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
