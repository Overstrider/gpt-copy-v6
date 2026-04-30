import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ApiErrorPayload, Conversation, Message, StreamHandlers } from "@/lib/api";

const api = vi.hoisted(() => {
  class MockApiError extends Error {
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

  return {
    ApiError: MockApiError,
    createConversation: vi.fn(),
    listConversations: vi.fn(),
    listMessages: vi.fn(),
    streamAssistantMessage: vi.fn()
  };
});

vi.mock("@/lib/api", () => ({
  ApiError: api.ApiError,
  createConversation: api.createConversation,
  listConversations: api.listConversations,
  listMessages: api.listMessages,
  streamAssistantMessage: api.streamAssistantMessage
}));

import { ApiError } from "@/lib/api";

import { useChatStream } from "./useChatStream";

const conversation: Conversation = {
  id: "conversation-1",
  title: "API design",
  created_at: "2026-04-28T12:00:00Z",
  updated_at: "2026-04-28T12:00:00Z"
};

const persistedUserMessage: Message = {
  id: "user-1",
  conversation_id: "conversation-1",
  role: "user",
  content: "Hello there",
  status: "completed",
  created_at: "2026-04-28T12:00:01Z",
  completed_at: "2026-04-28T12:00:01Z"
};

const assistantStartMessage: Message = {
  id: "assistant-1",
  conversation_id: "conversation-1",
  role: "assistant",
  content: "",
  status: "streaming",
  created_at: "2026-04-28T12:00:02Z",
  completed_at: null
};

const completedAssistantMessage: Message = {
  ...assistantStartMessage,
  content: "Persisted reply",
  status: "completed",
  completed_at: "2026-04-28T12:00:03Z"
};

function deferred() {
  let resolve: () => void = () => undefined;
  const promise = new Promise<void>((nextResolve) => {
    resolve = nextResolve;
  });

  return { promise, resolve };
}

function streamFailure(message: string): ApiErrorPayload {
  return { code: "provider_rate_limited", message };
}

describe("useChatStream", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads conversations and selected conversation messages", async () => {
    api.listConversations.mockResolvedValue([conversation]);
    api.listMessages.mockResolvedValue([persistedUserMessage]);

    const { result } = renderHook(() => useChatStream());

    await waitFor(() => {
      expect(result.current.isLoadingConversations).toBe(false);
      expect(result.current.selectedConversationId).toBe("conversation-1");
    });
    await waitFor(() => expect(result.current.messages).toEqual([persistedUserMessage]));

    expect(api.listConversations).toHaveBeenCalledTimes(1);
    expect(api.listMessages).toHaveBeenCalledWith("conversation-1");
  });

  it("optimistically streams a new conversation and reconciles persisted message ids", async () => {
    api.listConversations.mockResolvedValue([]);
    api.createConversation.mockResolvedValue(conversation);
    api.listMessages.mockResolvedValue([persistedUserMessage, completedAssistantMessage]);
    const stream = deferred();
    let handlers: StreamHandlers | undefined;
    api.streamAssistantMessage.mockImplementation(
      (_conversationId: string, _content: string, nextHandlers: StreamHandlers) => {
        handlers = nextHandlers;
        return stream.promise;
      }
    );

    const { result } = renderHook(() => useChatStream());

    await waitFor(() => expect(result.current.isLoadingConversations).toBe(false));

    let sendPromise: Promise<void> | undefined;
    act(() => {
      sendPromise = result.current.send("  Hello there  ");
    });

    await waitFor(() => expect(api.streamAssistantMessage).toHaveBeenCalled());
    await waitFor(() => {
      expect(result.current.messages).toEqual([
        expect.objectContaining({
          id: expect.stringMatching(/^local-user-/),
          role: "user",
          content: "Hello there"
        })
      ]);
    });

    act(() => {
      handlers?.onMessageStart?.(assistantStartMessage);
      handlers?.onDelta?.("Persisted ");
      handlers?.onDelta?.("reply");
      handlers?.onMessageComplete?.(completedAssistantMessage);
      stream.resolve();
    });

    await waitFor(() => expect(result.current.isStreaming).toBe(false));
    await sendPromise;

    expect(result.current.messages).toEqual([persistedUserMessage, completedAssistantMessage]);
    expect(result.current.messages.filter((message) => message.content === "Hello there")).toHaveLength(1);
  });

  it("cancels an active stream without showing an error", async () => {
    api.listConversations.mockResolvedValue([conversation]);
    api.listMessages.mockResolvedValue([]);
    const stream = deferred();
    let signal: AbortSignal | undefined;
    api.streamAssistantMessage.mockImplementation(
      (_conversationId: string, _content: string, _handlers: StreamHandlers, nextSignal?: AbortSignal) => {
        signal = nextSignal;
        return stream.promise;
      }
    );

    const { result } = renderHook(() => useChatStream());

    await waitFor(() => expect(result.current.selectedConversationId).toBe("conversation-1"));

    act(() => {
      void result.current.send("Cancel me");
    });

    await waitFor(() => expect(result.current.isStreaming).toBe(true));
    const cancelStream = (result.current as typeof result.current & { cancelStream?: () => void }).cancelStream;

    expect(cancelStream).toEqual(expect.any(Function));

    act(() => {
      cancelStream?.();
    });

    expect(signal?.aborted).toBe(true);
    await waitFor(() => expect(result.current.isStreaming).toBe(false));
    expect(result.current.error).toBeNull();
  });

  it("displays structured stream errors and re-enables sending", async () => {
    api.listConversations.mockResolvedValue([conversation]);
    api.listMessages.mockResolvedValue([]);
    api.streamAssistantMessage.mockImplementation(
      async (_conversationId: string, _content: string, handlers: StreamHandlers) => {
        handlers.onError?.(streamFailure("Provider unavailable"));
        throw new ApiError("provider_rate_limited", "Provider unavailable", 429);
      }
    );

    const { result } = renderHook(() => useChatStream());

    await waitFor(() => expect(result.current.selectedConversationId).toBe("conversation-1"));

    await act(async () => {
      await result.current.send("Try stream");
    });

    expect(result.current.error).toBe("Provider unavailable");
    expect(result.current.isStreaming).toBe(false);
  });
});
