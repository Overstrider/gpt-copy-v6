import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ChatApp } from "./ChatApp";

const conversation = {
  id: "conversation-1",
  title: "API design",
  created_at: "2026-04-28T12:00:00Z",
  updated_at: "2026-04-28T12:00:00Z"
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

describe("ChatApp", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    process.env.NEXT_PUBLIC_API_BASE_URL = "http://localhost:8080";
  });

  it("loads conversations and streams an assistant response after sending a message", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "http://localhost:8080/conversations" && method === "GET") {
        return new Response(JSON.stringify({ conversations: [conversation] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "http://localhost:8080/conversations/conversation-1/messages" && method === "GET") {
        return new Response(JSON.stringify({ messages: [] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (
        url === "http://localhost:8080/conversations/conversation-1/messages/stream" &&
        method === "POST"
      ) {
        return streamResponse([
          'event: message_start\ndata: {"message":{"id":"assistant-1","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:01Z"}}\n\n',
          'event: delta\ndata: {"content":"Hello "}\n\n',
          'event: delta\ndata: {"content":"from markdown"}\n\n',
          'event: message_complete\ndata: {"message":{"id":"assistant-1","conversation_id":"conversation-1","role":"assistant","content":"Hello from markdown","status":"completed","created_at":"2026-04-28T12:00:01Z","completed_at":"2026-04-28T12:00:02Z"}}\n\n'
        ]);
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(await screen.findByRole("button", { name: /api design/i })).toBeInTheDocument();

    await userEvent.type(screen.getByRole("textbox", { name: /message/i }), "Can you help?");
    await userEvent.click(screen.getByRole("button", { name: /send message/i }));

    expect(await screen.findByText("Can you help?")).toBeInTheDocument();
    expect(await screen.findByText("Hello from markdown")).toBeInTheDocument();

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:8080/conversations/conversation-1/messages/stream",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ content: "Can you help?" })
        })
      );
    });
  });

  it("shows a load error when the conversations request fails", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ error: { code: "backend_down", message: "Backend unavailable" } }), {
        status: 503,
        headers: { "content-type": "application/json" }
      })
    );

    render(<ChatApp />);

    expect(await screen.findByText("Backend unavailable")).toBeInTheDocument();
  });
});
