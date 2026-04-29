import type { z } from "zod";

import { getApiBaseUrl } from "./env";
import {
  ApiErrorResponseSchema,
  ConversationResponseSchema,
  ConversationsResponseSchema,
  MessagesResponseSchema,
  SendMessageResponseSchema,
  StreamDeltaSchema,
  StreamErrorSchema,
  StreamMessageCompleteSchema,
  StreamMessageStartSchema,
  type ApiErrorPayload,
  type Conversation,
  type Message
} from "./schemas";

export class ApiValidationError extends Error {
  constructor(
    message: string,
    public readonly issues: z.ZodIssue[]
  ) {
    super(message);
    this.name = "ApiValidationError";
  }
}

export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly status?: number,
    public readonly details?: unknown
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export type StreamHandlers = {
  onMessageStart?: (message: Message) => void;
  onDelta?: (content: string) => void;
  onMessageComplete?: (message: Message) => void;
  onError?: (error: ApiErrorPayload) => void;
};

function apiUrl(path: string) {
  return `${getApiBaseUrl()}${path}`;
}

async function readJson(response: Response) {
  const text = await response.text();

  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text) as unknown;
  } catch {
    throw new ApiError("invalid_json", "The API returned invalid JSON.", response.status);
  }
}

async function parseResponse<TSchema extends z.ZodType>(
  response: Response,
  schema: TSchema
): Promise<z.infer<TSchema>> {
  const payload = await readJson(response);

  if (!response.ok) {
    const parsed = ApiErrorResponseSchema.safeParse(payload);

    if (parsed.success) {
      throw new ApiError(
        parsed.data.error.code,
        parsed.data.error.message,
        response.status,
        parsed.data.error.details
      );
    }

    throw new ApiError("request_failed", `Request failed with status ${response.status}.`, response.status);
  }

  const parsed = schema.safeParse(payload);

  if (!parsed.success) {
    throw new ApiValidationError("The API response did not match the expected shape.", parsed.error.issues);
  }

  return parsed.data;
}

async function requestJson<TSchema extends z.ZodType>(
  path: string,
  schema: TSchema,
  init?: RequestInit
): Promise<z.infer<TSchema>> {
  const response = await fetch(apiUrl(path), {
    ...init,
    headers: {
      "content-type": "application/json",
      ...init?.headers
    }
  });

  return parseResponse(response, schema);
}

export async function listConversations() {
  const response = await requestJson("/conversations", ConversationsResponseSchema);
  return response.conversations;
}

export async function createConversation(title?: string) {
  const response = await requestJson("/conversations", ConversationResponseSchema, {
    method: "POST",
    body: JSON.stringify(title === undefined ? {} : { title })
  });
  return response.conversation;
}

export async function listMessages(conversationId: string) {
  const response = await requestJson(
    `/conversations/${encodeURIComponent(conversationId)}/messages`,
    MessagesResponseSchema
  );
  return response.messages;
}

export async function sendMessage(conversationId: string, content: string) {
  return requestJson(
    `/conversations/${encodeURIComponent(conversationId)}/messages`,
    SendMessageResponseSchema,
    {
      method: "POST",
      body: JSON.stringify({ content })
    }
  );
}

export async function streamAssistantMessage(
  conversationId: string,
  content: string,
  handlers: StreamHandlers,
  signal?: AbortSignal
) {
  const response = await fetch(apiUrl(`/conversations/${encodeURIComponent(conversationId)}/messages/stream`), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify({ content }),
    signal
  });

  if (!response.ok) {
    await parseResponse(response, SendMessageResponseSchema);
    return;
  }

  if (!response.body) {
    throw new ApiError("empty_stream", "The API did not return a stream.", response.status);
  }

  let sawTerminalEvent = false;

  await readEventStream(response.body, (event) => {
    if (!event.data) {
      return;
    }

    const payload = JSON.parse(event.data) as unknown;

    if (event.type === "message_start") {
      const parsed = StreamMessageStartSchema.safeParse(payload);
      if (!parsed.success) {
        throw new ApiValidationError("The stream message_start event was malformed.", parsed.error.issues);
      }
      handlers.onMessageStart?.(parsed.data.message);
      return;
    }

    if (event.type === "delta") {
      const parsed = StreamDeltaSchema.safeParse(payload);
      if (!parsed.success) {
        throw new ApiValidationError("The stream delta event was malformed.", parsed.error.issues);
      }
      handlers.onDelta?.(parsed.data.content);
      return;
    }

    if (event.type === "message_complete") {
      const parsed = StreamMessageCompleteSchema.safeParse(payload);
      if (!parsed.success) {
        throw new ApiValidationError("The stream message_complete event was malformed.", parsed.error.issues);
      }
      sawTerminalEvent = true;
      handlers.onMessageComplete?.(parsed.data.message);
      return;
    }

    if (event.type === "error") {
      const parsed = StreamErrorSchema.safeParse(payload);
      if (!parsed.success) {
        throw new ApiValidationError("The stream error event was malformed.", parsed.error.issues);
      }
      handlers.onError?.(parsed.data.error);
      throw new ApiError(parsed.data.error.code, parsed.data.error.message, response.status, parsed.data.error.details);
    }
  });

  if (!sawTerminalEvent) {
    throw new ApiError("stream_incomplete", "The API stream ended before completion.", response.status);
  }
}

type StreamEvent = {
  type: string;
  data: string;
};

async function readEventStream(body: ReadableStream<Uint8Array>, onEvent: (event: StreamEvent) => void) {
  const reader = body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    buffer = drainEvents(buffer, onEvent);
  }

  buffer += decoder.decode();
  drainEvents(`${buffer}\n\n`, onEvent);
}

function drainEvents(buffer: string, onEvent: (event: StreamEvent) => void) {
  let remaining = buffer.replace(/\r\n/g, "\n");
  let separator = remaining.indexOf("\n\n");

  while (separator >= 0) {
    const rawEvent = remaining.slice(0, separator);
    remaining = remaining.slice(separator + 2);

    if (rawEvent.trim()) {
      onEvent(parseEvent(rawEvent));
    }

    separator = remaining.indexOf("\n\n");
  }

  return remaining;
}

function parseEvent(rawEvent: string): StreamEvent {
  let type = "message";
  const data: string[] = [];

  for (const line of rawEvent.split("\n")) {
    if (line.startsWith("event:")) {
      type = line.slice("event:".length).trim();
    }

    if (line.startsWith("data:")) {
      data.push(line.slice("data:".length).trimStart());
    }
  }

  return {
    type,
    data: data.join("\n")
  };
}

export type { Conversation, Message };
