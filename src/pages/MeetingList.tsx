import { useEffect, useState, useCallback } from "react";
import type { Meeting } from "../types/index";
import { JA } from "../i18n/ja";
import { useTauri } from "../hooks/useTauri";
import { MeetingCard } from "../components/MeetingCard";

interface MeetingListProps {
  onNewRecording: () => void;
  onOpenMeeting: (meetingId: string) => void;
}

interface DeleteModal {
  isOpen: boolean;
  meetingId: string;
  meetingTitle: string;
}

const INITIAL_DELETE_MODAL: DeleteModal = {
  isOpen: false,
  meetingId: "",
  meetingTitle: "",
};

export function MeetingList({ onNewRecording, onOpenMeeting }: MeetingListProps) {
  const { listMeetings, deleteMeeting } = useTauri();
  const [meetings, setMeetings] = useState<Meeting[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [deleteModal, setDeleteModal] = useState<DeleteModal>(INITIAL_DELETE_MODAL);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const fetchMeetings = useCallback(async () => {
    setIsLoading(true);
    setFetchError(null);
    const result = await listMeetings();
    if (result.error !== null) {
      setFetchError(result.error);
    } else {
      setMeetings(result.data ?? []);
    }
    setIsLoading(false);
  }, [listMeetings]);

  useEffect(() => {
    void fetchMeetings();
  }, [fetchMeetings]);

  const openDeleteModal = useCallback((meeting: Meeting) => {
    setDeleteModal({
      isOpen: true,
      meetingId: meeting.id,
      meetingTitle: meeting.title,
    });
    setDeleteError(null);
  }, []);

  const closeDeleteModal = useCallback(() => {
    setDeleteModal(INITIAL_DELETE_MODAL);
    setDeleteError(null);
  }, []);

  const handleDeleteConfirm = useCallback(async () => {
    if (!deleteModal.meetingId) return;
    setIsDeleting(true);
    setDeleteError(null);
    const result = await deleteMeeting(deleteModal.meetingId);
    if (result.error !== null) {
      setDeleteError(result.error);
      setIsDeleting(false);
      return;
    }
    setMeetings((prev) => prev.filter((m) => m.id !== deleteModal.meetingId));
    setIsDeleting(false);
    setDeleteModal(INITIAL_DELETE_MODAL);
  }, [deleteModal.meetingId, deleteMeeting]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-20" role="status" aria-live="polite">
        <i className="fa-solid fa-spinner animate-spin text-2xl text-primary-500 mr-3" aria-hidden="true" />
        <span className="text-gray-600">{JA.common.loading}</span>
      </div>
    );
  }

  if (fetchError !== null) {
    return (
      <div className="flex flex-col items-center justify-center py-16 gap-4" role="alert">
        <i className="fa-solid fa-circle-exclamation text-3xl text-red-500" aria-hidden="true" />
        <p className="text-gray-700 text-sm">{fetchError}</p>
        <button
          type="button"
          onClick={() => void fetchMeetings()}
          className="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700 transition-colors"
          aria-label="会議一覧を再読み込み"
        >
          <i className="fa-solid fa-rotate-right mr-1.5" aria-hidden="true" />
          再読み込み
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <p className="text-sm text-gray-500">{meetings.length}件の会議</p>
        <button
          type="button"
          onClick={onNewRecording}
          aria-label={JA.meetingList.newButton}
          className="flex items-center gap-2 px-4 py-2 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors"
        >
          <i className="fa-solid fa-microphone" aria-hidden="true" />
          {JA.meetingList.newButton}
        </button>
      </div>

      {meetings.length === 0 ? (
        <div
          className="flex flex-col items-center justify-center py-20 text-center gap-4"
          role="status"
          aria-live="polite"
        >
          <i className="fa-solid fa-folder-open text-4xl text-gray-300" aria-hidden="true" />
          <p className="text-gray-500 text-sm">{JA.meetingList.empty}</p>
          <button
            type="button"
            onClick={onNewRecording}
            aria-label={JA.meetingList.newButton}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-primary-600 border border-primary-500 rounded-md hover:bg-primary-50 transition-colors"
          >
            <i className="fa-solid fa-plus" aria-hidden="true" />
            {JA.meetingList.newButton}
          </button>
        </div>
      ) : (
        <ul className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" aria-label="会議一覧">
          {meetings.map((meeting) => (
            <li key={meeting.id}>
              <MeetingCard
                meeting={meeting}
                onOpen={() => onOpenMeeting(meeting.id)}
                onDelete={() => openDeleteModal(meeting)}
              />
            </li>
          ))}
        </ul>
      )}

      {/* 削除確認モーダル */}
      {deleteModal.isOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-modal-title"
        >
          <div className="bg-white rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 flex flex-col gap-4">
            <div className="flex items-start gap-3">
              <i className="fa-solid fa-triangle-exclamation text-xl text-red-500 mt-0.5" aria-hidden="true" />
              <div className="flex flex-col gap-1">
                <h3 id="delete-modal-title" className="text-base font-semibold text-gray-900">
                  {JA.meetingList.deleteConfirmTitle}
                </h3>
                <p className="text-sm text-gray-600">
                  「{deleteModal.meetingTitle}」
                </p>
                <p className="text-sm text-gray-500">{JA.meetingList.deleteConfirmMessage}</p>
              </div>
            </div>

            {deleteError !== null && (
              <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
                {deleteError}
              </p>
            )}

            <div className="flex items-center gap-2 justify-end pt-1">
              <button
                type="button"
                onClick={closeDeleteModal}
                disabled={isDeleting}
                aria-label={JA.meetingList.deleteConfirmCancel}
                className="px-4 py-2 text-sm font-medium text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
              >
                {JA.meetingList.deleteConfirmCancel}
              </button>
              <button
                type="button"
                onClick={() => void handleDeleteConfirm()}
                disabled={isDeleting}
                aria-label={JA.meetingList.deleteConfirmOk}
                className="flex items-center gap-1.5 px-4 py-2 text-sm font-semibold text-white bg-red-600 hover:bg-red-700 rounded-md transition-colors disabled:opacity-50"
              >
                {isDeleting ? (
                  <i className="fa-solid fa-spinner animate-spin" aria-hidden="true" />
                ) : (
                  <i className="fa-solid fa-trash" aria-hidden="true" />
                )}
                {JA.meetingList.deleteConfirmOk}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
