import { render, screen, waitFor, within } from "@testing-library/react";
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

function controlledStreamResponse(chunks: string[]) {
  const encoder = new TextEncoder();
  let streamController!: ReadableStreamDefaultController<Uint8Array>;

  const response = new Response(
    new ReadableStream({
      start(controller) {
        streamController = controller;
        for (const chunk of chunks) {
          controller.enqueue(encoder.encode(chunk));
        }
      }
    }),
    {
      headers: { "content-type": "text/event-stream" }
    }
  );

  return {
    response,
    push(chunk: string) {
      streamController.enqueue(encoder.encode(chunk));
    },
    close() {
      streamController.close();
    }
  };
}

function jsonResponse(payload: unknown, init?: ResponseInit) {
  return new Response(JSON.stringify(payload), {
    ...init,
    headers: { "content-type": "application/json" }
  });
}

function deferredResponse() {
  let resolve: (response: Response) => void = () => undefined;
  const promise = new Promise<Response>((nextResolve) => {
    resolve = nextResolve;
  });

  return { promise, resolve };
}

function mockInitialChat(messages: unknown[] = []) {
  return vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
    const url = String(input);
    const method = init?.method ?? "GET";

    if (url === "http://localhost:8080/conversations" && method === "GET") {
      return jsonResponse({ conversations: [conversation] });
    }

    if (url === "http://localhost:8080/conversations/conversation-1/messages" && method === "GET") {
      return jsonResponse({ messages });
    }

    throw new Error(`Unexpected request: ${method} ${url}`);
  });
}

describe("ChatApp", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    process.env.NEXT_PUBLIC_API_BASE_URL = "http://localhost:8080";
  });

  it("shows loading states while conversations and selected messages are pending", async () => {
    // Keeps the loading contract visible for both initial app bootstrap and conversation transcript fetches.
    const conversations = deferredResponse();
    const messages = deferredResponse();

    vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "http://localhost:8080/conversations" && method === "GET") {
        return conversations.promise;
      }

      if (url === "http://localhost:8080/conversations/conversation-1/messages" && method === "GET") {
        return messages.promise;
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(screen.getByText("Loading conversations")).toBeInTheDocument();

    conversations.resolve(jsonResponse({ conversations: [conversation] }));

    expect(await screen.findByText("Loading messages")).toBeInTheDocument();

    messages.resolve(jsonResponse({ messages: [] }));

    expect(await screen.findByText("No messages in this conversation")).toBeInTheDocument();
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

  it("opens the mobile conversation drawer as a dismissible dialog", async () => {
    mockInitialChat();

    render(<ChatApp />);

    expect(await screen.findByRole("button", { name: /api design/i })).toBeInTheDocument();
    expect(screen.queryByRole("dialog", { name: /conversation sidebar/i })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /open sidebar/i }));

    const drawer = screen.getByRole("dialog", { name: /conversation sidebar/i });
    expect(within(drawer).getByRole("button", { name: /api design/i })).toBeInTheDocument();

    await userEvent.click(within(drawer).getByRole("button", { name: /collapse sidebar/i }));

    expect(screen.queryByRole("dialog", { name: /conversation sidebar/i })).not.toBeInTheDocument();
  });

  it("renders assistant GFM markdown without enabling raw HTML", async () => {
    // Covers the user-visible markdown contract while guarding against raw HTML rendering.
    mockInitialChat([
      {
        id: "user-markdown",
        conversation_id: "conversation-1",
        role: "user",
        content: "Show markdown",
        status: "completed",
        created_at: "2026-04-28T12:00:00Z",
        completed_at: "2026-04-28T12:00:00Z"
      },
      {
        id: "assistant-markdown",
        conversation_id: "conversation-1",
        role: "assistant",
        content: [
          "| Feature | Status |",
          "| --- | --- |",
          "| Tables | supported |",
          "",
          "- [x] Task lists",
          "",
          "~~finished~~",
          "",
          "<script>alert('x')</script>"
        ].join("\n"),
        status: "completed",
        created_at: "2026-04-28T12:00:01Z",
        completed_at: "2026-04-28T12:00:02Z"
      }
    ]);

    const { container } = render(<ChatApp />);

    expect(await screen.findByRole("table")).toBeInTheDocument();
    expect(screen.getByRole("checkbox")).toBeChecked();
    expect(screen.getByText("finished")).toBeInTheDocument();
    expect(container.querySelector("script")).toBeNull();
  });

  it("keeps the composer disabled while an assistant response is streaming", async () => {
    const stream = controlledStreamResponse([
      'event: message_start\ndata: {"message":{"id":"assistant-pending","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:01Z"}}\n\n',
      'event: delta\ndata: {"content":"Partial"}\n\n'
    ]);

    vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "http://localhost:8080/conversations" && method === "GET") {
        return jsonResponse({ conversations: [conversation] });
      }

      if (url === "http://localhost:8080/conversations/conversation-1/messages" && method === "GET") {
        return jsonResponse({ messages: [] });
      }

      if (
        url === "http://localhost:8080/conversations/conversation-1/messages/stream" &&
        method === "POST"
      ) {
        return stream.response;
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(await screen.findByRole("button", { name: /api design/i })).toBeInTheDocument();

    await userEvent.type(screen.getByRole("textbox", { name: /message/i }), "Keep waiting");
    await userEvent.click(screen.getByRole("button", { name: /send message/i }));

    expect(await screen.findByText("Partial")).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: /message/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /send message/i })).toBeDisabled();

    stream.push(
      'event: message_complete\ndata: {"message":{"id":"assistant-pending","conversation_id":"conversation-1","role":"assistant","content":"Partial","status":"completed","created_at":"2026-04-28T12:00:01Z","completed_at":"2026-04-28T12:00:02Z"}}\n\n'
    );
    stream.close();

    await waitFor(() => {
      expect(screen.getByRole("textbox", { name: /message/i })).not.toBeDisabled();
    });
  });

  it("creates a conversation before sending the first message from an empty state", async () => {
    const createdConversation = {
      ...conversation,
      id: "conversation-2",
      title: "First message"
    };
    const fetchMock = vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "http://localhost:8080/conversations" && method === "GET") {
        return new Response(JSON.stringify({ conversations: [] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "http://localhost:8080/conversations" && method === "POST") {
        return new Response(JSON.stringify({ conversation: createdConversation }), {
          status: 201,
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "http://localhost:8080/conversations/conversation-2/messages" && method === "GET") {
        return new Response(JSON.stringify({ messages: [] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (
        url === "http://localhost:8080/conversations/conversation-2/messages/stream" &&
        method === "POST"
      ) {
        return streamResponse([
          'event: message_start\ndata: {"message":{"id":"assistant-2","conversation_id":"conversation-2","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:01Z"}}\n\n',
          'event: delta\ndata: {"content":"Created"}\n\n',
          'event: message_complete\ndata: {"message":{"id":"assistant-2","conversation_id":"conversation-2","role":"assistant","content":"Created","status":"completed","created_at":"2026-04-28T12:00:01Z","completed_at":"2026-04-28T12:00:02Z"}}\n\n'
        ]);
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(await screen.findByText("No conversations yet.")).toBeInTheDocument();

    await userEvent.type(screen.getByRole("textbox", { name: /message/i }), "First message");
    await userEvent.click(screen.getByRole("button", { name: /send message/i }));

    expect((await screen.findAllByText("First message")).length).toBeGreaterThan(0);
    expect(await screen.findByText("Created")).toBeInTheDocument();

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:8080/conversations",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ title: "First message" })
        })
      );
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:8080/conversations/conversation-2/messages/stream",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ content: "First message" })
        })
      );
    });
  });

  it("shows streaming errors and re-enables sending", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
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
          'event: error\ndata: {"error":{"code":"provider_rate_limited","message":"Rate limited"}}\n\n'
        ]);
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(await screen.findByRole("button", { name: /api design/i })).toBeInTheDocument();

    await userEvent.type(screen.getByRole("textbox", { name: /message/i }), "Try stream");
    await userEvent.click(screen.getByRole("button", { name: /send message/i }));

    expect(await screen.findByText("Rate limited")).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByRole("textbox", { name: /message/i })).not.toBeDisabled();
    });
  });

  it("does not let an in-flight message load overwrite active streamed messages", async () => {
    let resolveMessages: (response: Response) => void = () => undefined;
    let delayedMessagesResolved = false;
    const pendingMessages = new Promise<Response>((resolve) => {
      resolveMessages = resolve;
    });
    vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url === "http://localhost:8080/conversations" && method === "GET") {
        return new Response(JSON.stringify({ conversations: [conversation] }), {
          headers: { "content-type": "application/json" }
        });
      }

      if (url === "http://localhost:8080/conversations/conversation-1/messages" && method === "GET") {
        const response = await pendingMessages;
        delayedMessagesResolved = true;
        return response;
      }

      if (
        url === "http://localhost:8080/conversations/conversation-1/messages/stream" &&
        method === "POST"
      ) {
        return streamResponse([
          'event: message_start\ndata: {"message":{"id":"assistant-race","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:01Z"}}\n\n',
          'event: delta\ndata: {"content":"Still here"}\n\n',
          'event: message_complete\ndata: {"message":{"id":"assistant-race","conversation_id":"conversation-1","role":"assistant","content":"Still here","status":"completed","created_at":"2026-04-28T12:00:01Z","completed_at":"2026-04-28T12:00:02Z"}}\n\n'
        ]);
      }

      throw new Error(`Unexpected request: ${method} ${url}`);
    });

    render(<ChatApp />);

    expect(await screen.findByRole("button", { name: /api design/i })).toBeInTheDocument();

    await userEvent.type(screen.getByRole("textbox", { name: /message/i }), "Race test");
    await userEvent.click(screen.getByRole("button", { name: /send message/i }));
    expect(await screen.findByText("Still here")).toBeInTheDocument();

    resolveMessages(
      new Response(JSON.stringify({
        messages: [
          {
            id: "stale-message",
            conversation_id: "conversation-1",
            role: "assistant",
            content: "Stale load",
            status: "completed",
            created_at: "2026-04-28T11:59:00Z",
            completed_at: "2026-04-28T11:59:01Z"
          }
        ]
      }), {
        headers: { "content-type": "application/json" }
      })
    );

    await waitFor(() => {
      expect(delayedMessagesResolved).toBe(true);
    });
    await waitFor(() => {
      expect(screen.getByText("Still here")).toBeInTheDocument();
      expect(screen.queryByText("Stale load")).not.toBeInTheDocument();
    });
  });
});
