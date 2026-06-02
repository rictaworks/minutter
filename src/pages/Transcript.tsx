import { useState, useCallback } from "react";
import { JA } from "../i18n/ja";
import { useTauri } from "../hooks/useTauri";
import type { GenerateResult } from "../types/index";

interface TranscriptProps {
  meetingId: string;
  rawText: string;
  onComplete: (result: GenerateResult) => void;
  onBack: () => void;
}

export function Transcript({ meetingId, rawText, onComplete, onBack }: TranscriptProps) {
  const { updateTranscript, generateAll } = useTauri();
  const [text, setText] = useState<string>(rawText);
  const [isSaving, setIsSaving] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [generateError, setGenerateError] = useState<string | null>(null);
  const [savedOk, setSavedOk] = useState(false);

  const handleSave = useCallback(async () => {
    setIsSaving(true);
    setSaveError(null);
    setSavedOk(false);
    const result = await updateTranscript(meetingId, text);
    if (result.error !== null) {
      setSaveError(result.error);
    } else {
      setSavedOk(true);
    }
    setIsSaving(false);
  }, [meetingId, text, updateTranscript]);

  const handleGenerate = useCallback(async () => {
    setIsGenerating(true);
    setGenerateError(null);
    const result = await generateAll(meetingId, text);
    if (result.error !== null) {
      setGenerateError(result.error);
      setIsGenerating(false);
      return;
    }
    setIsGenerating(false);
    if (result.data !== null) {
      onComplete(result.data);
    }
  }, [meetingId, text, generateAll, onComplete]);

  return (
    <div className="flex flex-col gap-5 max-w-3xl">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onBack}
          aria-label={JA.common.back}
          className="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 transition-colors"
        >
          <i className="fa-solid fa-chevron-left" aria-hidden="true" />
          {JA.common.back}
        </button>
      </div>

      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <label
            htmlFor="transcript-area"
            className="text-sm font-medium text-gray-700 flex items-center gap-2"
          >
            <i className="fa-solid fa-pen-to-square text-primary-500" aria-hidden="true" />
            {JA.transcript.title}
          </label>
          <span className="text-xs text-gray-400">
            {text.length} {JA.transcript.charCount}
          </span>
        </div>

        <textarea
          id="transcript-area"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={JA.transcript.placeholder}
          rows={16}
          className="w-full border border-gray-300 rounded-lg px-4 py-3 text-sm text-gray-800 leading-relaxed resize-y focus:outline-none focus:ring-2 focus:ring-primary-500 font-mono"
          aria-label={JA.transcript.title}
        />
      </div>

      {saveError !== null && (
        <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
          {saveError}
        </p>
      )}
      {savedOk && (
        <p className="text-sm text-green-700 bg-green-50 px-3 py-2 rounded" role="status">
          <i className="fa-solid fa-check mr-1.5" aria-hidden="true" />
          保存しました
        </p>
      )}
      {generateError !== null && (
        <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
          {generateError}
        </p>
      )}

      {isGenerating && (
        <div
          className="flex items-center gap-2 px-4 py-3 bg-primary-50 border border-primary-200 rounded-lg"
          role="status"
          aria-live="polite"
        >
          <i className="fa-solid fa-spinner animate-spin text-primary-600" aria-hidden="true" />
          <span className="text-sm font-medium text-primary-700">{JA.transcript.generating}</span>
        </div>
      )}

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={() => void handleGenerate()}
          disabled={isGenerating || isSaving || text.trim().length === 0}
          aria-label={JA.transcript.generateButton}
          className="flex items-center gap-2 px-5 py-2.5 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isGenerating ? (
            <i className="fa-solid fa-spinner animate-spin" aria-hidden="true" />
          ) : (
            <i className="fa-solid fa-rotate" aria-hidden="true" />
          )}
          {JA.transcript.generateButton}
        </button>
        <button
          type="button"
          onClick={() => void handleSave()}
          disabled={isSaving || isGenerating}
          aria-label={JA.transcript.saveButton}
          className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
        >
          {isSaving ? (
            <i className="fa-solid fa-spinner animate-spin" aria-hidden="true" />
          ) : (
            <i className="fa-solid fa-floppy-disk" aria-hidden="true" />
          )}
          {JA.transcript.saveButton}
        </button>
      </div>
    </div>
  );
}
