import { Component, lazy, Suspense, type ErrorInfo, type ReactNode } from "react";
import type { OfficeMapProps } from "./OfficeMap";

const OfficeMap = lazy(() =>
  import("./OfficeMap").then((module) => ({ default: module.OfficeMap }))
);

class OfficeMapErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("OfficeMap failed to render", error, info);
  }

  render() {
    if (this.state.failed) {
      return (
        <div className="border-b border-x-border px-4 py-2 text-xs text-x-text-secondary dark:border-x-border-dark">
          Office map is temporarily unavailable.
        </div>
      );
    }

    return this.props.children;
  }
}

export function SafeOfficeMap(props: OfficeMapProps) {
  return (
    <OfficeMapErrorBoundary>
      <Suspense
        fallback={
          <div className="border-b border-x-border px-4 py-2 text-xs text-x-text-secondary dark:border-x-border-dark">
            Loading office map...
          </div>
        }
      >
        <OfficeMap {...props} />
      </Suspense>
    </OfficeMapErrorBoundary>
  );
}
