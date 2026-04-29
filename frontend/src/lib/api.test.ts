import { beforeEach, describe, expect, it, vi } from "vitest";

import { ApiError, ApiValidationError, createConversation, listConversations, streamAssistantMessage } from "./api";

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

  it("rejects streams that end before a terminal event", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      streamResponse(['event: delta\ndata: {"content":"partial"}\n\n'])
    );

    await expect(streamAssistantMessage("conversation-1", "hello", {})).rejects.toMatchObject({
      code: "stream_incomplete"
    } satisfies Partial<ApiError>);
  });
});
