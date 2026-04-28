"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import {
  ApiError,
  createConversation,
  listConversations,
  listMessages,
  streamAssistantMessage,
  type Conversation,
  type Message
} from "@/lib/api";

function nowIso() {
  return new Date().toISOString();
}

function localMessage(conversationId: string, role: "user" | "assistant", content: string): Message {
  return {
    id: `local-${role}-${crypto.randomUUID()}`,
    conversation_id: conversationId,
    role,
    content,
    status: role === "assistant" ? "streaming" : "completed",
    created_at: nowIso(),
    completed_at: role === "assistant" ? null : nowIso()
  };
}

function titleFromMessage(content: string) {
  const trimmed = content.trim().replace(/\s+/g, " ");
  return trimmed.length > 48 ? `${trimmed.slice(0, 45)}...` : trimmed || "New chat";
}

function errorMessage(error: unknown) {
  if (error instanceof ApiError) {
    return error.message;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Something went wrong.";
}

export function useChatStream() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversationId, setSelectedConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoadingConversations, setIsLoadingConversations] = useState(true);
  const [isLoadingMessages, setIsLoadingMessages] = useState(false);
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const selectedConversation =
    conversations.find((conversation) => conversation.id === selectedConversationId) ?? null;

  useEffect(() => {
    let cancelled = false;

    async function loadConversations() {
      setIsLoadingConversations(true);
      setError(null);

      try {
        const loaded = await listConversations();
        if (cancelled) {
          return;
        }

        setConversations(loaded);
        setSelectedConversationId((current) => current ?? loaded[0]?.id ?? null);
      } catch (loadError) {
        if (!cancelled) {
          setError(errorMessage(loadError));
        }
      } finally {
        if (!cancelled) {
          setIsLoadingConversations(false);
        }
      }
    }

    loadConversations();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!selectedConversationId) {
      return;
    }

    let cancelled = false;
    const conversationId = selectedConversationId;

    async function loadMessages() {
      setIsLoadingMessages(true);
      setError(null);

      try {
        const loaded = await listMessages(conversationId);
        if (!cancelled) {
          setMessages(loaded);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(errorMessage(loadError));
        }
      } finally {
        if (!cancelled) {
          setIsLoadingMessages(false);
        }
      }
    }

    loadMessages();

    return () => {
      cancelled = true;
    };
  }, [selectedConversationId]);

  useEffect(() => {
    return () => {
      abortRef.current?.abort();
    };
  }, []);

  const startConversation = useCallback(async () => {
    setError(null);
    const conversation = await createConversation("New chat");
    setConversations((current) => [conversation, ...current.filter((item) => item.id !== conversation.id)]);
    setSelectedConversationId(conversation.id);
    setMessages([]);
  }, []);

  const selectConversation = useCallback((conversationId: string) => {
    setSelectedConversationId(conversationId);
  }, []);

  const send = useCallback(
    async (content: string) => {
      const trimmed = content.trim();

      if (!trimmed || isStreaming) {
        return;
      }

      setError(null);
      setIsStreaming(true);
      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;

      let conversation = selectedConversation;

      try {
        if (!conversation) {
          conversation = await createConversation(titleFromMessage(trimmed));
          setConversations((current) => [conversation!, ...current]);
          setSelectedConversationId(conversation.id);
        }

        const userMessage = localMessage(conversation.id, "user", trimmed);
        setMessages((current) => [...current, userMessage]);

        let assistantMessageId: string | null = null;

        await streamAssistantMessage(
          conversation.id,
          trimmed,
          {
            onMessageStart(message) {
              assistantMessageId = message.id;
              setMessages((current) => [...current, message]);
            },
            onDelta(delta) {
              setMessages((current) => {
                if (!assistantMessageId) {
                  const placeholder = localMessage(conversation!.id, "assistant", delta);
                  assistantMessageId = placeholder.id;
                  return [...current, placeholder];
                }

                return current.map((message) =>
                  message.id === assistantMessageId
                    ? { ...message, content: `${message.content}${delta}`, status: "streaming" }
                    : message
                );
              });
            },
            onMessageComplete(message) {
              assistantMessageId = message.id;
              setMessages((current) => {
                const hasMessage = current.some((item) => item.id === message.id);
                if (!hasMessage) {
                  return [...current, message];
                }

                return current.map((item) => (item.id === message.id ? message : item));
              });
            },
            onError(streamError) {
              setError(streamError.message);
            }
          },
          controller.signal
        );
      } catch (sendError) {
        if (!controller.signal.aborted) {
          setError(errorMessage(sendError));
        }
      } finally {
        if (abortRef.current === controller) {
          abortRef.current = null;
        }
        setIsStreaming(false);
      }
    },
    [isStreaming, selectedConversation]
  );

  return {
    conversations,
    selectedConversation,
    selectedConversationId,
    messages,
    isLoadingConversations,
    isLoadingMessages,
    isStreaming,
    error,
    send,
    selectConversation,
    startConversation
  };
}
