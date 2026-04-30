import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  ApiError,
  ApiValidationError,
  createConversation,
  listConversations,
  listMessages,
  sendMessage,
  streamAssistantMessage
} from "./api";
import { ConversationSchema, MessageSchema, SendMessageResponseSchema } from "./schemas";

const conversation = {
  id: "conversation-1",
  title: "API design",
  created_at: "2026-04-28T12:00:00Z",
  updated_at: "2026-04-28T12:00:00Z"
};

const userMessage = {
  id: "message-user-1",
  conversation_id: "conversation-1",
  role: "user",
  content: "Hello",
  status: "completed",
  created_at: "2026-04-28T12:00:01Z",
  completed_at: "2026-04-28T12:00:01Z"
};

const assistantMessage = {
  id: "message-assistant-1",
  conversation_id: "conversation-1",
  role: "assistant",
  content: "Hello back",
  status: "completed",
  created_at: "2026-04-28T12:00:02Z",
  completed_at: "2026-04-28T12:00:03Z"
};

function streamResponse(chunks: string[]) {
  const encoder = new TextEncoder();

  return new Response(
    new ReadableStream({
      start(controller) {
        for (const chunk of chunks) {
          controller.enqueue(encoder.encode(chunk));
        }
        controller.close();
      }
    }),
    {
      headers: { "content-type": "text/event-stream" }
    }
  );
}

describe("api schemas", () => {
  it("accepts valid conversation and message payloads and rejects malformed ones", () => {
    // Direct schema assertions keep API contract failures distinct from fetch transport behavior.
    expect(ConversationSchema.safeParse(conversation).success).toBe(true);
    expect(SendMessageResponseSchema.safeParse({
      user_message: userMessage,
      assistant_message: assistantMessage
    }).success).toBe(true);

    const invalidMessage = MessageSchema.safeParse({
      ...assistantMessage,
      role: "moderator",
      conversation_id: ""
    });

    expect(invalidMessage.success).toBe(false);
    if (!invalidMessage.success) {
      expect(invalidMessage.error.issues).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ path: ["role"] }),
          expect.objectContaining({ path: ["conversation_id"] })
        ])
      );
    }
  });
});

describe("api client", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    process.env.NEXT_PUBLIC_API_BASE_URL = "http://localhost:8080";
  });

  it("rejects malformed REST responses with a validation error", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ conversations: [{ id: 42 }] }), {
        headers: { "content-type": "application/json" }
      })
    );

    await expect(listConversations()).rejects.toBeInstanceOf(ApiValidationError);
  });

  it("preserves blank conversation titles so backend validation can reject them", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          conversation: {
            id: "conversation-1",
            title: "New conversation",
            created_at: "2026-04-28T12:00:00Z",
            updated_at: "2026-04-28T12:00:00Z"
          }
        }),
        {
          status: 201,
          headers: { "content-type": "application/json" }
        }
      )
    );

    await createConversation("");

    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/conversations",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ title: "" })
      })
    );
  });

  it("uses the configured backend base URL for conversation and message calls", async () => {
    process.env.NEXT_PUBLIC_API_BASE_URL = "https://backend.example.test/api/";
    const fetchMock = vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "https://backend.example.test/api/conversations" && method === "GET") {
        return new Response(JSON.stringify({ conversations: [conversation] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "https://backend.example.test/api/conversations" && method === "POST") {
        return new Response(JSON.stringify({ conversation }), {
          status: 201,
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "https://backend.example.test/api/conversations/conversation-1/messages" && method === "GET") {
        return new Response(JSON.stringify({ messages: [userMessage] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "https://backend.example.test/api/conversations/conversation-1/messages" && method === "POST") {
        return new Response(JSON.stringify({ user_message: userMessage, assistant_message: assistantMessage }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (
        url === "https://backend.example.test/api/conversations/conversation-1/messages/stream" &&
        method === "POST"
      ) {
        return streamResponse([
          'event: start\ndata: {"message":{"id":"message-assistant-1","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:02Z","completed_at":null}}\n\n',
          'event: delta\ndata: {"content":"Hello back"}\n\n',
          'event: complete\ndata: {"message":{"id":"message-assistant-1","conversation_id":"conversation-1","role":"assistant","content":"Hello back","status":"completed","created_at":"2026-04-28T12:00:02Z","completed_at":"2026-04-28T12:00:03Z"}}\n\n'
        ]);
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    await expect(listConversations()).resolves.toEqual([conversation]);
    await expect(createConversation("API design")).resolves.toEqual(conversation);
    await expect(listMessages("conversation-1")).resolves.toEqual([userMessage]);
    await expect(sendMessage("conversation-1", "Hello")).resolves.toEqual({
      user_message: userMessage,
      assistant_message: assistantMessage
    });
    await expect(streamAssistantMessage("conversation-1", "Hello", {})).resolves.toBeUndefined();

    expect(fetchMock).toHaveBeenCalledWith(
      "https://backend.example.test/api/conversations/conversation-1/messages/stream",
      expect.objectContaining({ method: "POST" })
    );
  });

  it("consumes backend stream events split across chunk boundaries", async () => {
    const events: string[] = [];
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      streamResponse([
        'event: sta',
        'rt\ndata: {"message":{"id":"message-assistant-1","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:02Z","completed_at":null}}\n',
        "\n",
        'event: delta\ndata: {"content":"Hello "}\n\n',
        'event: com',
        'plete\ndata: {"message":{"id":"message-assistant-1","conversation_id":"conversation-1","role":"assistant","content":"Hello back","status":"completed","created_at":"2026-04-28T12:00:02Z","completed_at":"2026-04-28T12:00:03Z"}}\n\n'
      ])
    );

    await streamAssistantMessage("conversation-1", "Hello", {
      onMessageStart: (message) => events.push(`start:${message.id}`),
      onDelta: (content) => events.push(`delta:${content}`),
      onMessageComplete: (message) => events.push(`complete:${message.content}`)
    });

    expect(events).toEqual(["start:message-assistant-1", "delta:Hello ", "complete:Hello back"]);
  });

  it("rejects streams that end before a terminal event", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      streamResponse(['event: delta\ndata: {"content":"partial"}\n\n'])
    );

    await expect(streamAssistantMessage("conversation-1", "hello", {})).rejects.toMatchObject({
      code: "stream_incomplete"
    } satisfies Partial<ApiError>);
  });
});
