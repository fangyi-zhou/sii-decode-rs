import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";
import DecodeWorker from "./decode.worker?worker";
import type { DecodeResponse } from "./decode.worker";

function App() {
  const [file, setFile] = useState<File | null>(null);
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);
  const workerRef = useRef<Worker | null>(null);

  // Initialize worker
  useEffect(() => {
    workerRef.current = new DecodeWorker();
    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  const handleFile = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      if (!event.target.files || event.target.files.length === 0) {
        return;
      }
      const selectedFile = event.target.files[0];
      if (textAreaRef.current) {
        // Clear the text area
        textAreaRef.current.value = "Decoding...";
      }
      if (downloadRef.current) {
        // Clear the download link
        if (downloadRef.current.href !== "#") {
          URL.revokeObjectURL(downloadRef.current.href);
        }
        downloadRef.current.href = "#";
      }
      setFile(selectedFile);
    },
    [setFile, textAreaRef, downloadRef],
  );

  useEffect(() => {
    if (!file || !workerRef.current) {
      return;
    }

    const worker = workerRef.current;

    const handleMessage = (event: MessageEvent<DecodeResponse>) => {
      if (event.data.type === "success") {
        const { result, blobUrl } = event.data;
        if (textAreaRef.current) {
          // Only show preview for large files to avoid UI freeze
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
      // Transfer the buffer instead of copying
      worker.postMessage({ type: "decode", buffer: arrayBuffer }, [
        arrayBuffer,
      ]);
    };
    reader.readAsArrayBuffer(file);

    return () => {
      worker.removeEventListener("message", handleMessage);
    };
  }, [file]);

  return (
    <>
      <h1>SII Decode</h1>
      <p>Select a SII file to decode</p>
      <div>
        <input
          type="file"
          id="file"
          data-testid="file-upload"
          onChange={handleFile}
        />
      </div>
      <br />
      <textarea
        id="output"
        rows={20}
        cols={50}
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
    </>
  );
}

export default App;
