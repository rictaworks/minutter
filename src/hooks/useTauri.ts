import { invoke } from "@tauri-apps/api/core";
import type {
  Meeting,
  MeetingDetail,
  GenerateResult,
} from "../types/index";

export interface TauriResult<T> {
  data: T | null;
  error: string | null;
}

function isTauriAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<TauriResult<T>> {
  if (!isTauriAvailable()) {
    return {
      data: null,
      error: "Tauri 環境外では実行できません",
    };
  }
  try {
    const data = await invoke<T>(command, args);
    return { data, error: null };
  } catch (err) {
    const message =
      err instanceof Error ? err.message : typeof err === "string" ? err : "不明なエラー";
    return { data: null, error: message };
  }
}

export function useTauri() {
  const checkModel = (): Promise<TauriResult<boolean>> =>
    safeInvoke<boolean>("check_model");

  const initApp = (): Promise<TauriResult<void>> =>
    safeInvoke<void>("init_app");

  const listAudioDevices = (): Promise<TauriResult<string[]>> =>
    safeInvoke<string[]>("list_audio_devices");

  const startRecording = (
    deviceIndex: number,
    meetingId: string
  ): Promise<TauriResult<void>> =>
    safeInvoke<void>("start_recording", { deviceIndex, meetingId });

  const stopRecording = (meetingId: string): Promise<TauriResult<string>> =>
    safeInvoke<string>("stop_recording", { meetingId });

  const importAudio = (
    path: string,
    meetingId: string
  ): Promise<TauriResult<string>> =>
    safeInvoke<string>("import_audio", { path, meetingId });

  const generateAll = (
    meetingId: string,
    text: string
  ): Promise<TauriResult<GenerateResult>> =>
    safeInvoke<GenerateResult>("generate_all", { meetingId, text });

  const createMeeting = (title: string): Promise<TauriResult<string>> =>
    safeInvoke<string>("create_meeting", { title });

  const listMeetings = (): Promise<TauriResult<Meeting[]>> =>
    safeInvoke<Meeting[]>("list_meetings");

  const getMeeting = (id: string): Promise<TauriResult<MeetingDetail | null>> =>
    safeInvoke<MeetingDetail | null>("get_meeting", { id });

  const deleteMeeting = (id: string): Promise<TauriResult<void>> =>
    safeInvoke<void>("delete_meeting", { id });

  const updateTranscript = (
    meetingId: string,
    editedText: string
  ): Promise<TauriResult<void>> =>
    safeInvoke<void>("update_transcript", { meetingId, editedText });

  const updateTodoCheck = (
    id: string,
    isChecked: boolean
  ): Promise<TauriResult<void>> =>
    safeInvoke<void>("update_todo_check", { id, isChecked });

  const deleteTodo = (id: string): Promise<TauriResult<void>> =>
    safeInvoke<void>("delete_todo", { id });

  const addTodo = (
    meetingId: string,
    todoText: string
  ): Promise<TauriResult<void>> =>
    safeInvoke<void>("add_todo", { meetingId, todoText });

  return {
    checkModel,
    initApp,
    listAudioDevices,
    startRecording,
    stopRecording,
    importAudio,
    generateAll,
    createMeeting,
    listMeetings,
    getMeeting,
    deleteMeeting,
    updateTranscript,
    updateTodoCheck,
    deleteTodo,
    addTodo,
  };
}
