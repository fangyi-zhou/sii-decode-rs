import { test, expect } from "vitest";
import App from "../src/App";
import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

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
