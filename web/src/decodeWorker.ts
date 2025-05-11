import { decode } from "sii-decode-rs";

type OkResponse = {
  status: "ok";
  decoded: string;
};

type ErrorResponse = {
  status: "error";
  error: string;
};

type Response = OkResponse | ErrorResponse;

export const run = (data: Uint8Array): Response => {
  // console.log("Did I ever get here?");
  // return { status: "ok", decoded: "mock" };
  try {
    const decoded = decode(data);
    return { decoded, status: "ok" };
  } catch (error) {
    if (error instanceof Error) {
      return { error: error.message, status: "error" };
    }
    return { error: "Unknown error", status: "error" };
  }
};
