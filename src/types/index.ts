export type MeetingStatus = "recording" | "processing" | "done" | "error";
export type SectionType = "decisions" | "next" | "body";

export interface Meeting {
  id: string;
  title: string;
  recorded_at: string; // ISO 8601
  audio_path: string;
  duration_sec: number;
  status: MeetingStatus;
  created_at: string;
}

export interface Transcript {
  id: string;
  meeting_id: string;
  raw_text: string;
  edited_text: string;
  vosk_confidence: number;
  created_at: string;
  updated_at: string;
}

export interface Minute {
  id: string;
  meeting_id: string;
  section_type: SectionType;
  content: string;
  sort_order: number;
  created_at: string;
}

export interface Todo {
  id: string;
  meeting_id: string;
  todo_text: string;
  due_keyword: string;
  is_checked: boolean;
  is_manual: boolean;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface Summary {
  id: string;
  meeting_id: string;
  summary_text: string;
  created_at: string;
}

export interface MeetingDetail {
  meeting: Meeting;
  transcript: Transcript | null;
  minutes: Minute[];
  todos: Todo[];
  summary: Summary | null;
}

export interface GenerateResult {
  minutes: MinuteItem[];
  todos: TodoItem[];
  summary: string;
}

export interface MinuteItem {
  section_type: SectionType;
  content: string;
  sort_order: number;
}

export interface TodoItem {
  todo_text: string;
  due_keyword: string;
}

export const MEETING_STATUSES: MeetingStatus[] = [
  "recording",
  "processing",
  "done",
  "error",
];

export const SECTION_TYPES: SectionType[] = ["decisions", "next", "body"];
