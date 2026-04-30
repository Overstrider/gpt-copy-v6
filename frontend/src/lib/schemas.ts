import { z } from "zod";

export const ErrorPayloadSchema = z.object({
  code: z.string().min(1),
  message: z.string().min(1),
  details: z.unknown().optional()
});

export const ApiErrorResponseSchema = z.object({
  error: ErrorPayloadSchema
});

export const ConversationSchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  created_at: z.string().min(1),
  updated_at: z.string().min(1)
});

export const MessageRoleSchema = z.enum(["user", "assistant", "system"]);

export const MessageSchema = z.object({
  id: z.string().min(1),
  conversation_id: z.string().min(1),
  role: MessageRoleSchema,
  content: z.string(),
  status: z.string().min(1),
  created_at: z.string().min(1),
  completed_at: z.string().min(1).optional().nullable()
});

export const ConversationsResponseSchema = z.object({
  conversations: z.array(ConversationSchema)
});

export const ConversationResponseSchema = z.object({
  conversation: ConversationSchema
});

export const MessagesResponseSchema = z.object({
  messages: z.array(MessageSchema)
});

export const SendMessageResponseSchema = z.object({
  user_message: MessageSchema,
  assistant_message: MessageSchema
});

export const StreamMessageStartSchema = z.object({
  message: MessageSchema
});

export const StreamDeltaSchema = z.object({
  content: z.string()
});

export const StreamMessageCompleteSchema = z.object({
  message: MessageSchema
});

export const StreamErrorSchema = z.object({
  error: ErrorPayloadSchema
});

export type ApiErrorPayload = z.infer<typeof ErrorPayloadSchema>;
export type Conversation = z.infer<typeof ConversationSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type MessageRole = z.infer<typeof MessageRoleSchema>;
