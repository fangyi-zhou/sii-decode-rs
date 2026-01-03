import { test, expect, vi } from "vitest";
import App from "../src/App";
import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// Mock the worker module
vi.mock("../src/decode.worker?worker", () => {
  return {
    default: class MockWorker {
      onmessage = null;
      onerror = null;
      onmessageerror = null;

      postMessage(data) {
        // simulate async worker response
        setTimeout(() => {
          if (data.type === "decode") {
            const bytes = new Uint8Array(data.buffer);
            const text = new TextDecoder().decode(bytes);

            // check if valid sii
            if (text.startsWith("SiiN")) {
              const blob = new Blob([text], { type: "text/plain" });
              const blobUrl = URL.createObjectURL(blob);
              this.onmessage?.({
                data: { type: "success", result: text, blobUrl },
              });
            } else {
              this.onmessage?.({
                data: { type: "error", message: "Invalid SII file format" },
              });
            }
          }
        }, 10);
      }

      addEventListener(type, handler) {
        if (type === "message") {
          this.onmessage = handler;
        }
      }

      removeEventListener(type, handler) {
        if (type === "message" && this.onmessage === handler) {
          this.onmessage = null;
        }
      }

      terminate() {}
    },
  };
});

test("App can render", () => {
  render(<App />);
  // Has a file upload box
  expect(screen.getByTestId("file-upload")).toBeInTheDocument();

  // Has a download button
  const downloadButton = screen.getByTestId("file-download");
  expect(downloadButton).toBeInTheDocument();
  expect(downloadButton).toHaveAttribute("href", "#");

  // Has a read only text area for displaying
  const textArea = screen.getByTestId("file-display");
  expect(textArea).toBeInTheDocument();
  expect(textArea).toHaveAttribute("readonly");
  // Initially empty
  expect(textArea).toHaveValue("");
});

test("App can decode", async () => {
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["SiiN"], "test.sii");
  userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  // Wait for the text area to be updated
  await waitFor(() => {
    expect(textArea).toHaveValue("SiiN");

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).not.toHaveAttribute("href", "#");
  });
});

test("App can display error", async () => {
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["invalid"], "test.sii");
  userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  // Wait for the text area to be updated
  await waitFor(() => {
    expect(textArea).toHaveDisplayValue(/Error:/);

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).toHaveAttribute("href", "#");
  });
});
