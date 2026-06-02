import type { Meeting, MeetingStatus } from "../types/index";
import { JA } from "../i18n/ja";

interface MeetingCardProps {
  meeting: Meeting;
  onOpen: () => void;
  onDelete: () => void;
}

function formatJapaneseDate(isoString: string): string {
  const d = new Date(isoString);
  const year = d.getFullYear();
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hours = String(d.getHours()).padStart(2, "0");
  const minutes = String(d.getMinutes()).padStart(2, "0");
  return `${year}年${month}月${day}日 ${hours}:${minutes}`;
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}分${String(s).padStart(2, "0")}秒`;
}

interface StatusBadgeConfig {
  label: string;
  className: string;
}

const STATUS_BADGE_CONFIG: Record<MeetingStatus, StatusBadgeConfig> = {
  recording: {
    label: JA.status.recording,
    className: "bg-blue-100 text-blue-700 border border-blue-300",
  },
  processing: {
    label: JA.status.processing,
    className: "bg-yellow-100 text-yellow-700 border border-yellow-300",
  },
  done: {
    label: JA.status.done,
    className: "bg-green-100 text-green-700 border border-green-300",
  },
  error: {
    label: JA.status.error,
    className: "bg-red-100 text-red-700 border border-red-300",
  },
};

export function MeetingCard({ meeting, onOpen, onDelete }: MeetingCardProps) {
  const badge = STATUS_BADGE_CONFIG[meeting.status];

  return (
    <article
      className="bg-white rounded-lg border border-gray-200 shadow-sm p-4 flex flex-col gap-3"
      aria-label={`会議: ${meeting.title}`}
    >
      <div className="flex items-start justify-between gap-2">
        <h2 className="text-base font-semibold text-gray-800 leading-snug flex-1 truncate">
          {meeting.title}
        </h2>
        <span
          className={`text-xs px-2 py-0.5 rounded-full font-medium flex-shrink-0 ${badge.className}`}
          aria-label={`ステータス: ${badge.label}`}
        >
          {badge.label}
        </span>
      </div>

      <div className="flex flex-col gap-1 text-sm text-gray-500">
        <span>
          <i className="fa-regular fa-calendar mr-1" aria-hidden="true" />
          {formatJapaneseDate(meeting.recorded_at)}
        </span>
        {meeting.duration_sec > 0 && (
          <span>
            <i className="fa-regular fa-clock mr-1" aria-hidden="true" />
            {formatDuration(meeting.duration_sec)}
          </span>
        )}
      </div>

      <div className="flex items-center gap-2 justify-end pt-1 border-t border-gray-100">
        <button
          type="button"
          onClick={onOpen}
          aria-label={`${meeting.title}を開く`}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors"
        >
          <i className="fa-solid fa-folder-open" aria-hidden="true" />
          {JA.meetingList.openButton}
        </button>
        <button
          type="button"
          onClick={onDelete}
          aria-label={`${meeting.title}を削除`}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-danger-600 border border-danger-500 hover:bg-danger-50 rounded-md transition-colors"
        >
          <i className="fa-solid fa-trash" aria-hidden="true" />
          {JA.meetingList.deleteButton}
        </button>
      </div>
    </article>
  );
}
