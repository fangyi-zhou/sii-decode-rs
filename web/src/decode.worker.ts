import { decode } from "sii-decode-rs";

export type DecodeRequest = {
  type: "decode";
  buffer: ArrayBuffer;
};

export type DecodeResponse =
  | { type: "success"; result: string; blobUrl: string }
  | { type: "error"; message: string };

self.onmessage = (event: MessageEvent<DecodeRequest>) => {
  if (event.data.type === "decode") {
    try {
      const bytes = new Uint8Array(event.data.buffer);
      const result = decode(bytes);

      // create blob URL in worker to avoid transferring large string back
      const blob = new Blob([result], { type: "text/plain" });
      const blobUrl = URL.createObjectURL(blob);
      self.postMessage({
        type: "success",
        result,
        blobUrl,
      } satisfies DecodeResponse);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      self.postMessage({
        type: "error",
        message,
      } satisfies DecodeResponse);
    }
  }
};
