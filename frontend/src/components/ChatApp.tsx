"use client";

import {
  AlertCircle,
  LoaderCircle,
  Menu,
  MessageSquare,
  PanelLeftClose,
  Plus,
  Send,
  Sparkles
} from "lucide-react";
import { FormEvent, useMemo, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { useChatStream } from "@/hooks/useChatStream";
import type { Conversation, Message } from "@/lib/api";

export function ChatApp() {
  const {
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
  } = useChatStream();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  const sidebar = (
    <Sidebar
      conversations={conversations}
      selectedConversationId={selectedConversationId}
      isLoading={isLoadingConversations}
      onSelect={(conversationId) => {
        selectConversation(conversationId);
        setIsSidebarOpen(false);
      }}
      onNewChat={() => {
        startConversation().catch(() => undefined);
        setIsSidebarOpen(false);
      }}
      onClose={() => setIsSidebarOpen(false)}
    />
  );

  return (
    <main className="flex h-dvh overflow-hidden bg-[#f7f7f4] text-neutral-950">
      <div className="hidden w-80 shrink-0 border-r border-neutral-200 bg-[#ededdf] md:block">
        {sidebar}
      </div>

      {isSidebarOpen ? (
        <div className="fixed inset-0 z-40 flex md:hidden">
          <button
            type="button"
            aria-label="Close sidebar"
            className="absolute inset-0 bg-black/30"
            onClick={() => setIsSidebarOpen(false)}
          />
          <div className="relative z-10 w-[82vw] max-w-80 border-r border-neutral-200 bg-[#ededdf] shadow-xl">
            {sidebar}
          </div>
        </div>
      ) : null}

      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 shrink-0 items-center gap-3 border-b border-neutral-200 bg-white/90 px-3 backdrop-blur md:px-5">
          <button
            type="button"
            aria-label="Open sidebar"
            className="inline-flex h-10 w-10 items-center justify-center rounded-md text-neutral-700 hover:bg-neutral-100 md:hidden"
            onClick={() => setIsSidebarOpen(true)}
          >
            <Menu className="h-5 w-5" aria-hidden="true" />
          </button>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium text-neutral-900">
              {selectedConversation?.title ?? "gpt-copy-v6"}
            </p>
            <p className="text-xs text-neutral-500">
              {isStreaming ? "Assistant responding" : "Ready"}
            </p>
          </div>
        </header>

        {error ? <ErrorBanner message={error} /> : null}

        <Transcript
          messages={messages}
          isLoading={isLoadingMessages}
          isStreaming={isStreaming}
          hasConversation={Boolean(selectedConversation)}
        />

        <Composer onSend={send} disabled={isStreaming || isLoadingConversations} />
      </section>
    </main>
  );
}

function Sidebar({
  conversations,
  selectedConversationId,
  isLoading,
  onSelect,
  onNewChat,
  onClose
}: {
  conversations: Conversation[];
  selectedConversationId: string | null;
  isLoading: boolean;
  onSelect: (conversationId: string) => void;
  onNewChat: () => void;
  onClose: () => void;
}) {
  return (
    <aside className="flex h-full flex-col">
      <div className="flex h-14 items-center gap-2 px-3">
        <button
          type="button"
          onClick={onNewChat}
          className="inline-flex h-10 flex-1 items-center justify-center gap-2 rounded-md bg-neutral-950 px-3 text-sm font-medium text-white hover:bg-neutral-800"
        >
          <Plus className="h-4 w-4" aria-hidden="true" />
          New chat
        </button>
        <button
          type="button"
          aria-label="Close sidebar"
          onClick={onClose}
          className="inline-flex h-10 w-10 items-center justify-center rounded-md text-neutral-700 hover:bg-black/5 md:hidden"
        >
          <PanelLeftClose className="h-5 w-5" aria-hidden="true" />
        </button>
      </div>

      <nav aria-label="Conversations" className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {isLoading ? (
          <div className="flex items-center gap-2 px-3 py-4 text-sm text-neutral-600">
            <LoaderCircle className="h-4 w-4 animate-spin" aria-hidden="true" />
            Loading conversations
          </div>
        ) : null}

        {!isLoading && conversations.length === 0 ? (
          <p className="px-3 py-4 text-sm text-neutral-600">No conversations yet.</p>
        ) : null}

        <div className="space-y-1">
          {conversations.map((conversation) => {
            const selected = conversation.id === selectedConversationId;
            return (
              <button
                type="button"
                key={conversation.id}
                onClick={() => onSelect(conversation.id)}
                className={`flex h-11 w-full items-center gap-2 rounded-md px-3 text-left text-sm transition ${
                  selected
                    ? "bg-white text-neutral-950 shadow-sm"
                    : "text-neutral-700 hover:bg-white/60 hover:text-neutral-950"
                }`}
              >
                <MessageSquare className="h-4 w-4 shrink-0" aria-hidden="true" />
                <span className="min-w-0 flex-1 truncate">{conversation.title}</span>
              </button>
            );
          })}
        </div>
      </nav>
    </aside>
  );
}

function ErrorBanner({ message }: { message: string }) {
  return (
    <div role="alert" className="border-b border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800">
      <div className="mx-auto flex max-w-3xl items-start gap-2">
        <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
        <span>{message}</span>
      </div>
    </div>
  );
}

function Transcript({
  messages,
  isLoading,
  isStreaming,
  hasConversation
}: {
  messages: Message[];
  isLoading: boolean;
  isStreaming: boolean;
  hasConversation: boolean;
}) {
  const showStreamingStatus = isStreaming && !messages.some((message) => message.role === "assistant");

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto flex min-h-full w-full max-w-3xl flex-col px-4 py-6 md:px-6">
        {isLoading ? (
          <div className="flex flex-1 items-center justify-center text-sm text-neutral-500">
            <LoaderCircle className="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
            Loading messages
          </div>
        ) : null}

        {!isLoading && messages.length === 0 ? <EmptyState hasConversation={hasConversation} /> : null}

        {!isLoading && messages.length > 0 ? (
          <div className="space-y-5">
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
          </div>
        ) : null}

        {showStreamingStatus ? (
          <div className="mt-5 flex items-center gap-2 text-sm text-neutral-500">
            <LoaderCircle className="h-4 w-4 animate-spin" aria-hidden="true" />
            Assistant is typing
          </div>
        ) : null}
      </div>
    </div>
  );
}

function EmptyState({ hasConversation }: { hasConversation: boolean }) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <div className="flex h-11 w-11 items-center justify-center rounded-md bg-emerald-100 text-emerald-700">
        <Sparkles className="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <h1 className="text-lg font-semibold text-neutral-950">
          {hasConversation ? "No messages in this conversation" : "gpt-copy-v6"}
        </h1>
        <p className="mt-1 text-sm text-neutral-500">Start with a message below.</p>
      </div>
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === "user";
  const content = useMemo(() => message.content || (message.status === "streaming" ? " " : ""), [
    message.content,
    message.status
  ]);

  return (
    <article className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div
        className={`max-w-[92%] rounded-md px-4 py-3 text-sm leading-6 md:max-w-[78%] ${
          isUser
            ? "bg-emerald-700 text-white"
            : "border border-neutral-200 bg-white text-neutral-900 shadow-sm"
        }`}
      >
        {isUser ? (
          <p className="whitespace-pre-wrap">{content}</p>
        ) : (
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              p: ({ children }) => <p className="mb-3 last:mb-0">{children}</p>,
              ul: ({ children }) => <ul className="mb-3 list-disc pl-5 last:mb-0">{children}</ul>,
              ol: ({ children }) => <ol className="mb-3 list-decimal pl-5 last:mb-0">{children}</ol>,
              code: ({ children }) => (
                <code className="rounded bg-neutral-100 px-1 py-0.5 text-[0.92em] text-neutral-900">
                  {children}
                </code>
              ),
              pre: ({ children }) => (
                <pre className="mb-3 overflow-x-auto rounded-md bg-neutral-950 p-3 text-sm text-white last:mb-0">
                  {children}
                </pre>
              )
            }}
          >
            {content}
          </ReactMarkdown>
        )}
      </div>
    </article>
  );
}

function Composer({
  onSend,
  disabled
}: {
  onSend: (content: string) => Promise<void>;
  disabled: boolean;
}) {
  const [content, setContent] = useState("");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = content.trim();

    if (!trimmed || disabled) {
      return;
    }

    setContent("");
    await onSend(trimmed);
  }

  return (
    <div className="shrink-0 border-t border-neutral-200 bg-white px-3 py-3 md:px-5">
      <form
        onSubmit={handleSubmit}
        className="mx-auto flex max-w-3xl items-end gap-2 rounded-md border border-neutral-300 bg-white p-2 shadow-sm focus-within:border-neutral-500"
      >
        <textarea
          aria-label="Message"
          rows={1}
          value={content}
          onChange={(event) => setContent(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              event.currentTarget.form?.requestSubmit();
            }
          }}
          placeholder="Message gpt-copy-v6"
          className="max-h-36 min-h-10 flex-1 resize-none bg-transparent px-2 py-2 text-sm text-neutral-950 outline-none placeholder:text-neutral-400"
          disabled={disabled}
        />
        <button
          type="submit"
          aria-label="Send message"
          disabled={disabled || !content.trim()}
          className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-neutral-950 text-white hover:bg-neutral-800 disabled:cursor-not-allowed disabled:bg-neutral-300"
        >
          {disabled ? (
            <LoaderCircle className="h-4 w-4 animate-spin" aria-hidden="true" />
          ) : (
            <Send className="h-4 w-4" aria-hidden="true" />
          )}
        </button>
      </form>
    </div>
  );
}
