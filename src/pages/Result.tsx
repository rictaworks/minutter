import { useEffect, useState, useCallback } from "react";
import { JA } from "../i18n/ja";
import { useTauri } from "../hooks/useTauri";
import type { MeetingDetail, SectionType, Todo } from "../types/index";
import { TodoItem } from "../components/TodoItem";

type ResultTab = "minutes" | "todos" | "summary";

interface ResultProps {
  meetingId: string;
  onBack: () => void;
}

const SECTION_LABEL: Record<SectionType, string> = {
  decisions: JA.result.decisionsTitle,
  next: JA.result.nextTitle,
  body: JA.result.bodyTitle,
};

export function Result({ meetingId, onBack }: ResultProps) {
  const { getMeeting, updateTodoCheck, deleteTodo, addTodo } = useTauri();
  const [activeTab, setActiveTab] = useState<ResultTab>("minutes");
  const [detail, setDetail] = useState<MeetingDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [newTodoText, setNewTodoText] = useState("");
  const [isAddingTodo, setIsAddingTodo] = useState(false);
  const [todoError, setTodoError] = useState<string | null>(null);

  const fetchDetail = useCallback(async () => {
    setIsLoading(true);
    setFetchError(null);
    const result = await getMeeting(meetingId);
    if (result.error !== null) {
      setFetchError(result.error);
    } else {
      setDetail(result.data ?? null);
    }
    setIsLoading(false);
  }, [meetingId, getMeeting]);

  useEffect(() => {
    void fetchDetail();
  }, [fetchDetail]);

  const handleTodoCheck = useCallback(
    async (todoId: string, checked: boolean) => {
      const result = await updateTodoCheck(todoId, checked);
      if (result.error !== null) {
        setTodoError(result.error);
        return;
      }
      setDetail((prev) => {
        if (prev === null) return prev;
        return {
          ...prev,
          todos: prev.todos.map((t) =>
            t.id === todoId ? { ...t, is_checked: checked } : t
          ),
        };
      });
    },
    [updateTodoCheck]
  );

  const handleTodoDelete = useCallback(
    async (todoId: string) => {
      const result = await deleteTodo(todoId);
      if (result.error !== null) {
        setTodoError(result.error);
        return;
      }
      setDetail((prev) => {
        if (prev === null) return prev;
        return {
          ...prev,
          todos: prev.todos.map((t) =>
            t.id === todoId ? { ...t, is_deleted: true } : t
          ),
        };
      });
    },
    [deleteTodo]
  );

  const handleAddTodo = useCallback(async () => {
    const trimmed = newTodoText.trim();
    if (trimmed.length === 0) return;
    setIsAddingTodo(true);
    setTodoError(null);
    const result = await addTodo(meetingId, trimmed);
    if (result.error !== null) {
      setTodoError(result.error);
      setIsAddingTodo(false);
      return;
    }
    setIsAddingTodo(false);
    setNewTodoText("");
    // 再取得してリストを更新
    await fetchDetail();
  }, [meetingId, newTodoText, addTodo, fetchDetail]);

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
          onClick={() => void fetchDetail()}
          className="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700 transition-colors"
          aria-label="生成結果を再読み込み"
        >
          <i className="fa-solid fa-rotate-right mr-1.5" aria-hidden="true" />
          再読み込み
        </button>
      </div>
    );
  }

  const minutes = detail?.minutes ?? [];
  const todos = (detail?.todos ?? []).filter((t: Todo) => !t.is_deleted);
  const summaryText = detail?.summary?.summary_text ?? "";

  const decisionsMinutes = minutes.filter((m) => m.section_type === "decisions");
  const nextMinutes = minutes.filter((m) => m.section_type === "next");
  const bodyMinutes = minutes.filter((m) => m.section_type === "body");

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

      {detail && (
        <h2 className="text-lg font-semibold text-gray-800">{detail.meeting.title}</h2>
      )}

      {/* タブ */}
      <div className="flex border-b border-gray-200" role="tablist" aria-label="結果タブ">
        <button
          type="button"
          role="tab"
          id="tab-minutes"
          aria-selected={activeTab === "minutes"}
          aria-controls="panel-minutes"
          onClick={() => setActiveTab("minutes")}
          className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "minutes"
              ? "border-primary-600 text-primary-700"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          <i className="fa-solid fa-file-lines" aria-hidden="true" />
          {JA.result.tabMinutes}
        </button>
        <button
          type="button"
          role="tab"
          id="tab-todos"
          aria-selected={activeTab === "todos"}
          aria-controls="panel-todos"
          onClick={() => setActiveTab("todos")}
          className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "todos"
              ? "border-primary-600 text-primary-700"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          <i className="fa-solid fa-list-check" aria-hidden="true" />
          {JA.result.tabTodos}
        </button>
        <button
          type="button"
          role="tab"
          id="tab-summary"
          aria-selected={activeTab === "summary"}
          aria-controls="panel-summary"
          onClick={() => setActiveTab("summary")}
          className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "summary"
              ? "border-primary-600 text-primary-700"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          <i className="fa-solid fa-align-left" aria-hidden="true" />
          {JA.result.tabSummary}
        </button>
      </div>

      {/* 議事録タブ */}
      {activeTab === "minutes" && (
        <div
          id="panel-minutes"
          role="tabpanel"
          aria-labelledby="tab-minutes"
          className="flex flex-col gap-6"
        >
          {minutes.length === 0 ? (
            <p className="text-sm text-gray-500 py-8 text-center">{JA.result.emptyMinutes}</p>
          ) : (
            <>
              {decisionsMinutes.length > 0 && (
                <section aria-labelledby="section-decisions">
                  <h3
                    id="section-decisions"
                    className="text-sm font-semibold text-gray-700 mb-2 flex items-center gap-2"
                  >
                    <span className="w-2 h-2 rounded-full bg-green-500 inline-block" aria-hidden="true" />
                    {SECTION_LABEL.decisions}
                  </h3>
                  <ul className="flex flex-col gap-2">
                    {decisionsMinutes.map((m) => (
                      <li
                        key={m.id}
                        className="text-sm text-gray-800 bg-green-50 border border-green-100 rounded-md px-4 py-2.5 leading-relaxed"
                      >
                        {m.content}
                      </li>
                    ))}
                  </ul>
                </section>
              )}

              {nextMinutes.length > 0 && (
                <section aria-labelledby="section-next">
                  <h3
                    id="section-next"
                    className="text-sm font-semibold text-gray-700 mb-2 flex items-center gap-2"
                  >
                    <span className="w-2 h-2 rounded-full bg-blue-500 inline-block" aria-hidden="true" />
                    {SECTION_LABEL.next}
                  </h3>
                  <ul className="flex flex-col gap-2">
                    {nextMinutes.map((m) => (
                      <li
                        key={m.id}
                        className="text-sm text-gray-800 bg-blue-50 border border-blue-100 rounded-md px-4 py-2.5 leading-relaxed"
                      >
                        {m.content}
                      </li>
                    ))}
                  </ul>
                </section>
              )}

              {bodyMinutes.length > 0 && (
                <section aria-labelledby="section-body">
                  <h3
                    id="section-body"
                    className="text-sm font-semibold text-gray-700 mb-2 flex items-center gap-2"
                  >
                    <span className="w-2 h-2 rounded-full bg-gray-400 inline-block" aria-hidden="true" />
                    {SECTION_LABEL.body}
                  </h3>
                  <ul className="flex flex-col gap-2">
                    {bodyMinutes.map((m) => (
                      <li
                        key={m.id}
                        className="text-sm text-gray-800 bg-gray-50 border border-gray-100 rounded-md px-4 py-2.5 leading-relaxed"
                      >
                        {m.content}
                      </li>
                    ))}
                  </ul>
                </section>
              )}
            </>
          )}
        </div>
      )}

      {/* ToDoタブ */}
      {activeTab === "todos" && (
        <div
          id="panel-todos"
          role="tabpanel"
          aria-labelledby="tab-todos"
          className="flex flex-col gap-4"
        >
          {todoError !== null && (
            <p className="text-sm text-red-600 bg-red-50 px-3 py-2 rounded" role="alert">
              {todoError}
            </p>
          )}

          {todos.length === 0 ? (
            <p className="text-sm text-gray-500 py-8 text-center">{JA.result.emptyTodos}</p>
          ) : (
            <ul className="flex flex-col divide-y divide-gray-100" aria-label="ToDoリスト">
              {todos.map((todo) => (
                <TodoItem
                  key={todo.id}
                  todo={todo}
                  onCheck={(checked) => void handleTodoCheck(todo.id, checked)}
                  onDelete={() => void handleTodoDelete(todo.id)}
                />
              ))}
            </ul>
          )}

          {/* 新規ToDo追加フォーム */}
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void handleAddTodo();
            }}
            className="flex items-center gap-2 mt-2"
            aria-label="新しいToDoを追加"
          >
            <label htmlFor="new-todo-input" className="sr-only">
              {JA.result.addTodoPlaceholder}
            </label>
            <input
              id="new-todo-input"
              type="text"
              value={newTodoText}
              onChange={(e) => setNewTodoText(e.target.value)}
              placeholder={JA.result.addTodoPlaceholder}
              disabled={isAddingTodo}
              className="flex-1 border border-gray-300 rounded-md px-3 py-2 text-sm text-gray-800 focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:bg-gray-100"
              aria-label={JA.result.addTodoPlaceholder}
            />
            <button
              type="submit"
              disabled={isAddingTodo || newTodoText.trim().length === 0}
              aria-label={JA.result.addTodoButton}
              className="flex items-center gap-1.5 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isAddingTodo ? (
                <i className="fa-solid fa-spinner animate-spin" aria-hidden="true" />
              ) : (
                <i className="fa-solid fa-plus" aria-hidden="true" />
              )}
              {JA.result.addTodoButton}
            </button>
          </form>
        </div>
      )}

      {/* 要約タブ */}
      {activeTab === "summary" && (
        <div
          id="panel-summary"
          role="tabpanel"
          aria-labelledby="tab-summary"
          className="flex flex-col gap-4"
        >
          {summaryText.length === 0 ? (
            <p className="text-sm text-gray-500 py-8 text-center">{JA.result.emptySummary}</p>
          ) : (
            <div className="bg-gray-50 border border-gray-200 rounded-lg px-5 py-4">
              <p className="text-sm text-gray-800 leading-relaxed whitespace-pre-wrap">
                {summaryText}
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
