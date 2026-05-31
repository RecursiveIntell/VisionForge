import { AppShell } from "./components/layout/AppShell";
import { ToastProvider } from "./components/shared/Toast";
import { ErrorBoundary } from "./components/shared/ErrorBoundary";
import { ConfigProvider } from "./context/ConfigContext";

function App() {
  return (
    <ErrorBoundary>
      <ToastProvider>
        <ConfigProvider>
          <AppShell />
        </ConfigProvider>
      </ToastProvider>
    </ErrorBoundary>
  );
}

export default App;
