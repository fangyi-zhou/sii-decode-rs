import { useEffect, useRef, useState } from "react";
import "./App.css";

function App() {
  const [file, setFile] = useState<File | null>(null);
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);

  const decodeWorker = new ComlinkWorker<typeof import("./decodeWorker")>(
    new URL("./decodeWorker.ts", import.meta.url),
    {
      type: "module",
    },
  );

  const handleFile = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files && event.target.files.length > 0) {
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
    }
  };

  useEffect(() => {
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        const arrayBuffer = e.target?.result as ArrayBuffer;
        const bytes = new Uint8Array(arrayBuffer);
        // console.log("Am I here?");
        decodeWorker.run(bytes).then((result) => {
          // console.log(result);
          if (result.status === "ok") {
            if (textAreaRef.current) {
              textAreaRef.current.value = result.decoded;
            }
            if (downloadRef.current) {
              const blob = new Blob([result.decoded], { type: "text/plain" });
              const url = URL.createObjectURL(blob);
              downloadRef.current.href = url;
              downloadRef.current.download = file.name.replace(
                ".sii",
                "-decoded.sii",
              );
            }
          } else {
            if (textAreaRef.current) {
              textAreaRef.current.value = result.error;
            }
          }
        });
      };
      // console.log("Am I here too?");
      reader.readAsArrayBuffer(file);
    }
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
      <p className="read-the-docs">
        Your file will not be uploaded to the server. It will be decoded in your
        browser.
      </p>
    </>
  );
}

export default App;
