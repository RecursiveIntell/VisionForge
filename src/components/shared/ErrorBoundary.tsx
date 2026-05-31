import { Component, ErrorInfo, ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Component crashed:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex min-h-screen items-center justify-center bg-zinc-900 p-4">
          <div className="max-w-md rounded-lg bg-zinc-800 p-6 shadow-xl">
            <h2 className="mb-4 text-xl font-bold text-red-400">
              Something went wrong
            </h2>
            <p className="mb-4 text-sm text-zinc-300">
              {this.state.error?.message || "An unexpected error occurred"}
            </p>
            <details className="mb-4 text-xs text-zinc-400">
              <summary className="cursor-pointer hover:text-zinc-200">
                Stack trace
              </summary>
              <pre className="mt-2 overflow-auto rounded bg-zinc-900 p-2">
                {this.state.error?.stack}
              </pre>
            </details>
            <button
              onClick={this.handleReset}
              className="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
            >
              Try again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
