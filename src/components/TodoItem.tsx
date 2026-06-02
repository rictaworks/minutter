import type { Todo } from "../types/index";

interface TodoItemProps {
  todo: Todo;
  onCheck: (checked: boolean) => void;
  onDelete: () => void;
}

export function TodoItem({ todo, onCheck, onDelete }: TodoItemProps) {
  if (todo.is_deleted) {
    return null;
  }

  return (
    <li
      className="flex items-center gap-3 py-2 px-3 rounded-md hover:bg-gray-50 transition-colors"
      aria-label={`ToDo: ${todo.todo_text}`}
    >
      <input
        type="checkbox"
        id={`todo-${todo.id}`}
        checked={todo.is_checked}
        onChange={(e) => onCheck(e.target.checked)}
        className="w-4 h-4 accent-primary-600 cursor-pointer flex-shrink-0"
        aria-label={`ToDo「${todo.todo_text}」を${todo.is_checked ? "未完了" : "完了"}にする`}
      />
      <label
        htmlFor={`todo-${todo.id}`}
        className={`flex-1 text-sm cursor-pointer leading-snug ${
          todo.is_checked ? "line-through text-gray-400" : "text-gray-800"
        }`}
      >
        {todo.todo_text}
        {todo.due_keyword && (
          <span className="ml-2 text-xs text-primary-600 bg-primary-50 px-1.5 py-0.5 rounded">
            {todo.due_keyword}
          </span>
        )}
      </label>
      <button
        type="button"
        onClick={onDelete}
        aria-label={`ToDo「${todo.todo_text}」を削除`}
        className="flex-shrink-0 p-1 text-gray-400 hover:text-danger-500 transition-colors rounded"
      >
        <i className="fa-solid fa-trash text-xs" aria-hidden="true" />
      </button>
    </li>
  );
}
