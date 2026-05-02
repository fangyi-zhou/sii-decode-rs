import { useCallback, useEffect, useRef, useState } from "react";
import type { ChangeEvent, ReactNode } from "react";
import "./App.css";
import DecodeWorker from "./decode.worker?worker";
import type { DecodeResponse } from "./decode.worker";

type Tab = "decode" | "tracker";

type TrackerAnalysis = {
  analytics: {
    delivery_count: number;
    total_distance_km: number;
    total_revenue: number;
    unique_cargos: string[];
    unique_companies: string[];
    job_type_breakdown: Record<string, number>;
    brand_distance_km: Record<string, number>;
    cargo_category_coverage: Record<string, number>;
  };
  achievements: Array<{
    id: string;
    display_name: string;
    description: string;
    status: "complete" | "in_progress";
    progress: {
      current: number;
      target: number;
      unit: string;
    };
    evidence: Array<{
      label: string;
      value: string;
      complete: boolean;
    }>;
  }>;
};

function App() {
  const [file, setFile] = useState<File | null>(null);
  const [trackerFile, setTrackerFile] = useState<File | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>(() =>
    window.location.hash === "#tracker" ? "tracker" : "decode",
  );
  const [trackerState, setTrackerState] = useState<{
    status: "idle" | "loading" | "success" | "error";
    analysis: TrackerAnalysis | null;
    error: string | null;
  }>({ status: "idle", analysis: null, error: null });
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);
  const workerRef = useRef<Worker | null>(null);

  useEffect(() => {
    workerRef.current = new DecodeWorker();
    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  useEffect(() => {
    const handleHashChange = () => {
      setActiveTab(window.location.hash === "#tracker" ? "tracker" : "decode");
    };
    window.addEventListener("hashchange", handleHashChange);
    return () => {
      window.removeEventListener("hashchange", handleHashChange);
    };
  }, []);

  const selectTab = useCallback((tab: Tab) => {
    setActiveTab(tab);
    window.location.hash = tab === "tracker" ? "tracker" : "decode";
  }, []);

  const handleFile = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files || event.target.files.length === 0) {
      return;
    }
    const selectedFile = event.target.files[0];
    if (textAreaRef.current) {
      textAreaRef.current.value = "Decoding...";
    }
    if (downloadRef.current) {
      if (downloadRef.current.href !== "#") {
        URL.revokeObjectURL(downloadRef.current.href);
      }
      downloadRef.current.href = "#";
    }
    setFile(selectedFile);
  }, []);

  useEffect(() => {
    if (!file || !workerRef.current) {
      return;
    }

    const worker = workerRef.current;

    const handleMessage = (event: MessageEvent<DecodeResponse>) => {
      if (event.data.type === "success") {
        const { result, blobUrl } = event.data;
        if (textAreaRef.current) {
          const PREVIEW_LIMIT = 100_000;
          if (result.length > PREVIEW_LIMIT) {
            textAreaRef.current.value =
              result.slice(0, PREVIEW_LIMIT) +
              `\n\n... (${(result.length / 1024 / 1024).toFixed(2)} MB total - download for full content)`;
          } else {
            textAreaRef.current.value = result;
          }
        }
        if (downloadRef.current) {
          downloadRef.current.href = blobUrl;
          downloadRef.current.download = file.name.replace(
            ".sii",
            "-decoded.sii",
          );
        }
      } else if (event.data.type === "error") {
        if (textAreaRef.current) {
          textAreaRef.current.value = `Error: ${event.data.message}`;
        }
      }
    };

    worker.addEventListener("message", handleMessage);

    const reader = new FileReader();
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer;
      worker.postMessage({ type: "decode", buffer: arrayBuffer }, [
        arrayBuffer,
      ]);
    };
    reader.readAsArrayBuffer(file);

    return () => {
      worker.removeEventListener("message", handleMessage);
    };
  }, [file]);

  const handleTrackerFile = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (!event.target.files || event.target.files.length === 0) {
        return;
      }
      setTrackerState({ status: "loading", analysis: null, error: null });
      setTrackerFile(event.target.files[0]);
    },
    [],
  );

  useEffect(() => {
    if (!trackerFile || !workerRef.current) {
      return;
    }

    const worker = workerRef.current;

    const handleMessage = (event: MessageEvent<DecodeResponse>) => {
      if (event.data.type === "analysis-success") {
        try {
          setTrackerState({
            status: "success",
            analysis: JSON.parse(event.data.result) as TrackerAnalysis,
            error: null,
          });
        } catch {
          setTrackerState({
            status: "error",
            analysis: null,
            error: "Tracker output could not be read.",
          });
        }
      } else if (event.data.type === "error") {
        setTrackerState({
          status: "error",
          analysis: null,
          error: event.data.message,
        });
      }
    };

    worker.addEventListener("message", handleMessage);

    const reader = new FileReader();
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer;
      worker.postMessage({ type: "analyze", buffer: arrayBuffer }, [
        arrayBuffer,
      ]);
    };
    reader.readAsArrayBuffer(trackerFile);

    return () => {
      worker.removeEventListener("message", handleMessage);
    };
  }, [trackerFile]);

  return (
    <main className="app-shell">
      <header className="app-header">
        <h1>SII Decode</h1>
        <nav className="tabs" aria-label="Primary">
          <button
            type="button"
            className={activeTab === "decode" ? "active" : ""}
            onClick={() => selectTab("decode")}
          >
            Decode
          </button>
          <button
            type="button"
            className={activeTab === "tracker" ? "active" : ""}
            onClick={() => selectTab("tracker")}
          >
            ETS2 Tracker
          </button>
        </nav>
      </header>

      {activeTab === "decode" ? (
        <section className="panel" aria-labelledby="decode-title">
          <h2 id="decode-title">Decode</h2>
          <div className="file-row">
            <input
              type="file"
              id="file"
              data-testid="file-upload"
              onChange={handleFile}
            />
          </div>
          <textarea
            id="output"
            rows={20}
            ref={textAreaRef}
            data-testid="file-display"
            spellCheck="false"
            readOnly
          />
          <div>
            <a href="#" ref={downloadRef} data-testid="file-download">
              Download decoded file
            </a>
          </div>
        </section>
      ) : (
        <TrackerView state={trackerState} onFileChange={handleTrackerFile} />
      )}

      <p className="footer">
        Your file is not uploaded to any server, it is decoded using your own
        browser.
        <br />
        This tools is{" "}
        <a href="https://github.com/fangyi-zhou/sii-decode-rs/">open source</a>.
        If you encounter any issues, please report them{" "}
        <a href="https://github.com/fangyi-zhou/sii-decode-rs/issues">
          on GitHub
        </a>
        .
      </p>
    </main>
  );
}

function TrackerView({
  state,
  onFileChange,
}: {
  state: {
    status: "idle" | "loading" | "success" | "error";
    analysis: TrackerAnalysis | null;
    error: string | null;
  };
  onFileChange: (event: ChangeEvent<HTMLInputElement>) => void;
}) {
  return (
    <section className="panel" aria-labelledby="tracker-title">
      <h2 id="tracker-title">ETS2 Tracker</h2>
      <div className="file-row">
        <input
          type="file"
          id="tracker-file"
          data-testid="tracker-file-upload"
          onChange={onFileChange}
        />
      </div>

      {state.status === "loading" ? (
        <p data-testid="tracker-status">Analyzing save...</p>
      ) : null}

      {state.status === "error" ? (
        <p className="error" data-testid="tracker-error">
          Error: {state.error}
        </p>
      ) : null}

      {state.status === "success" && state.analysis ? (
        <div className="tracker-results" data-testid="tracker-results">
          <AnalyticsSummary analysis={state.analysis} />
          <AchievementList achievements={state.analysis.achievements} />
        </div>
      ) : null}
    </section>
  );
}

function AnalyticsSummary({ analysis }: { analysis: TrackerAnalysis }) {
  const analytics = analysis.analytics;
  return (
    <section className="summary-grid" aria-label="Analytics summary">
      <Metric label="Deliveries" value={analytics.delivery_count} />
      <Metric label="Distance" value={`${analytics.total_distance_km} km`} />
      <Metric label="Revenue" value={formatCurrency(analytics.total_revenue)} />
      <Metric label="Cargos" value={analytics.unique_cargos.length} />
      <Metric label="Companies" value={analytics.unique_companies.length} />
      <Metric
        label="Job Types"
        value={Object.keys(analytics.job_type_breakdown).length}
      />
    </section>
  );
}

function Metric({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function AchievementList({
  achievements,
}: {
  achievements: TrackerAnalysis["achievements"];
}) {
  return (
    <section className="achievements" aria-label="Achievements">
      {achievements.map((achievement) => (
        <article className="achievement" key={achievement.id}>
          <div className="achievement-heading">
            <div>
              <h3>{achievement.display_name}</h3>
              <p>{achievement.description}</p>
            </div>
            <span className={`status ${achievement.status}`}>
              {achievement.status === "complete" ? "Complete" : "In progress"}
            </span>
          </div>
          <progress
            value={achievement.progress.current}
            max={achievement.progress.target}
            aria-label={`${achievement.display_name} progress`}
          />
          <div className="progress-text">
            {achievement.progress.current} / {achievement.progress.target}{" "}
            {achievement.progress.unit}
          </div>
          <ul className="evidence-list">
            {achievement.evidence.map((evidence) => (
              <li key={evidence.label}>
                <span>{evidence.label}</span>
                <span>{evidence.value}</span>
              </li>
            ))}
          </ul>
        </article>
      ))}
    </section>
  );
}

function formatCurrency(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "EUR",
    maximumFractionDigits: 0,
  }).format(value);
}

export default App;
