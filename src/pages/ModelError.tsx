import { JA } from "../i18n/ja";

interface ModelErrorProps {
  onRetry: () => void;
}

export function ModelError({ onRetry }: ModelErrorProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-full py-16 px-4">
      <div className="bg-white rounded-2xl border border-red-200 shadow-md p-10 max-w-lg w-full flex flex-col items-center gap-6 text-center">
        <div className="w-16 h-16 rounded-full bg-red-100 flex items-center justify-center">
          <i
            className="fa-solid fa-triangle-exclamation text-3xl text-red-500"
            aria-hidden="true"
          />
        </div>

        <div className="flex flex-col gap-2">
          <h2 className="text-xl font-bold text-gray-900">{JA.modelError.title}</h2>
          <p className="text-sm text-gray-600 leading-relaxed">
            {JA.modelError.description}
          </p>
        </div>

        <a
          href={JA.modelError.downloadUrl}
          target="_blank"
          rel="noopener noreferrer"
          aria-label={JA.modelError.downloadLabel}
          className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-primary-600 border border-primary-500 rounded-md hover:bg-primary-50 transition-colors"
        >
          <i className="fa-solid fa-arrow-down" aria-hidden="true" />
          {JA.modelError.downloadLabel}
        </a>

        <button
          type="button"
          onClick={onRetry}
          aria-label={JA.modelError.retryButton}
          className="flex items-center gap-2 px-6 py-2.5 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors"
        >
          <i className="fa-solid fa-rotate-right" aria-hidden="true" />
          {JA.modelError.retryButton}
        </button>
      </div>
    </div>
  );
}
