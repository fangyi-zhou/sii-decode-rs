import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";
import { decode } from "sii-decode-rs";

function App() {
  const [file, setFile] = useState<File | null>(null);
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);

  const handleFile = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files || event.target.files.length === 0) {
      return;
    }
    const selectedFile = event.target.files[0];
    if (textAreaRef.current) {
      // Clear the text area
      textAreaRef.current.value = "";
    }
    if (downloadRef.current) {
      // Clear the download link
      if (downloadRef.current.href !== "#") {
        URL.revokeObjectURL(downloadRef.current.href);
      }
      downloadRef.current.href = "#";
    }
    setFile(selectedFile);
  }, [setFile, textAreaRef, downloadRef]);

  useEffect(() => {
    if (!file) {
      return;
    }
    const reader = new FileReader();
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer;
      const bytes = new Uint8Array(arrayBuffer);
      try {
        const decoded = decode(bytes);
        if (textAreaRef.current) {
          textAreaRef.current.value = decoded;
        }
        if (downloadRef.current) {
          const blob = new Blob([decoded], { type: "text/plain" });
          const url = URL.createObjectURL(blob);
          downloadRef.current.href = url;
          downloadRef.current.download = file.name.replace(
            ".sii",
            "-decoded.sii",
          );
        }
      } catch (error) {
        if (textAreaRef.current && error instanceof Error) {
          textAreaRef.current.value = error.toString();
        }
      }
    };
    reader.readAsArrayBuffer(file);
  }, [file]);

  return (
    <>
      <h1>SII Decode (beta)</h1>
      <p>Select your file</p>
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
        readOnly
      />
      <div>
        <a href="#" ref={downloadRef} data-testid="file-download">
          Download
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
