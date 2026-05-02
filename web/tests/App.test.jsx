import { test, expect, vi, beforeEach } from "vitest";
import App from "../src/App";
import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("../src/decode.worker?worker", () => {
  return {
    default: class MockWorker {
      onmessage = null;
      onerror = null;
      onmessageerror = null;
      messageListeners = new Set();

      postMessage(data) {
        setTimeout(() => {
          if (data.type === "decode") {
            const bytes = new Uint8Array(data.buffer);
            const text = new TextDecoder().decode(bytes);

            if (text.startsWith("SiiN")) {
              const blob = new Blob([text], { type: "text/plain" });
              const blobUrl = URL.createObjectURL(blob);
              this.dispatchMessage({
                data: { type: "success", result: text, blobUrl },
              });
            } else {
              this.dispatchMessage({
                data: { type: "error", message: "Invalid SII file format" },
              });
            }
          } else if (data.type === "analyze") {
            const bytes = new Uint8Array(data.buffer);
            const text = new TextDecoder().decode(bytes);

            if (text.startsWith("BSII")) {
              this.dispatchMessage({
                data: {
                  type: "analysis-success",
                  result: JSON.stringify({
                    analytics: {
                      delivery_count: 2,
                      total_distance_km: 1844,
                      total_revenue: 21760.5,
                      unique_cargos: ["gravel", "canned_beef"],
                      unique_companies: ["lkwlog", "stokes", "transinet"],
                      job_type_breakdown: { cargo: 1, quick: 1 },
                      brand_distance_km: { Mercedes: 362, Scania: 1482 },
                      cargo_category_coverage: { bulk: 1, refrigerated: 1 },
                    },
                    achievements: [
                      {
                        id: "experience_beats_all",
                        display_name: "Experience Beats All",
                        description:
                          "Complete deliveries with all trailer types.",
                        status: "in_progress",
                        progress: {
                          current: 1,
                          target: 8,
                          unit: "categories",
                        },
                        evidence: [
                          {
                            label: "Bulk cargo",
                            value: "gravel",
                            complete: true,
                          },
                        ],
                      },
                    ],
                  }),
                },
              });
            } else {
              this.dispatchMessage({
                data: {
                  type: "error",
                  message:
                    "Structured BSII analysis requires a binary BSII file",
                },
              });
            }
          }
        }, 10);
      }

      dispatchMessage(event) {
        this.onmessage?.(event);
        this.messageListeners.forEach((handler) => handler(event));
      }

      addEventListener(type, handler) {
        if (type === "message") {
          this.messageListeners.add(handler);
        }
      }

      removeEventListener(type, handler) {
        if (type === "message") {
          this.messageListeners.delete(handler);
        }
      }

      terminate() {}
    },
  };
});

beforeEach(() => {
  window.location.hash = "";
});

test("App can render", () => {
  render(<App />);

  expect(screen.getByTestId("file-upload")).toBeInTheDocument();

  const downloadButton = screen.getByTestId("file-download");
  expect(downloadButton).toBeInTheDocument();
  expect(downloadButton).toHaveAttribute("href", "#");

  const textArea = screen.getByTestId("file-display");
  expect(textArea).toBeInTheDocument();
  expect(textArea).toHaveAttribute("readonly");
  expect(textArea).toHaveValue("");
});

test("App can decode", async () => {
  window.location.hash = "#decode";
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["SiiN"], "test.sii");
  await userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  await waitFor(() => {
    expect(textArea).toHaveValue("SiiN");

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).not.toHaveAttribute("href", "#");
  });
});

test("App can display error", async () => {
  window.location.hash = "#decode";
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["invalid"], "test.sii");
  await userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  await waitFor(() => {
    expect(textArea).toHaveDisplayValue(/Error:/);

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).toHaveAttribute("href", "#");
  });
});

test("Tracker can render analysis", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["BSII"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });
  expect(screen.getByText("Experience Beats All")).toBeInTheDocument();
  expect(screen.getByText("1 / 8 categories")).toBeInTheDocument();
  const deliveries = screen.getByText("Deliveries").closest(".metric");
  expect(deliveries).not.toBeNull();
  expect(within(deliveries).getByText("2")).toBeInTheDocument();
});

test("Tracker can display unsupported-format error", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["SiiN"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-error")).toHaveTextContent(
      /Structured BSII analysis/,
    );
  });
});

test("Tracker can display worker error", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["invalid"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-error")).toHaveTextContent(/Error:/);
  });
});
