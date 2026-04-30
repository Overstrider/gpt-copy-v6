import { expect, test } from "@playwright/test";

const conversation = {
  id: "conversation-1",
  title: "Playwright chat",
  created_at: "2026-04-28T12:00:00Z",
  updated_at: "2026-04-28T12:00:00Z"
};

test("user can send a message and see a streamed assistant response", async ({ page }) => {
  await page.route("http://api.test/conversations", async (route) => {
    if (route.request().method() === "GET") {
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify({ conversations: [conversation] })
      });
      return;
    }

    await route.fallback();
  });

  await page.route("http://api.test/conversations/conversation-1/messages", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ messages: [] })
    });
  });

  await page.route("http://api.test/conversations/conversation-1/messages/stream", async (route) => {
    await expect(route.request().postDataJSON()).toEqual({ content: "Write a haiku" });
    await route.fulfill({
      contentType: "text/event-stream",
      body: [
        'event: message_start\ndata: {"message":{"id":"assistant-1","conversation_id":"conversation-1","role":"assistant","content":"","status":"streaming","created_at":"2026-04-28T12:00:01Z"}}',
        "",
        'event: delta\ndata: {"content":"Mocked "}',
        "",
        'event: delta\ndata: {"content":"assistant reply"}',
        "",
        'event: message_complete\ndata: {"message":{"id":"assistant-1","conversation_id":"conversation-1","role":"assistant","content":"Mocked assistant reply","status":"completed","created_at":"2026-04-28T12:00:01Z","completed_at":"2026-04-28T12:00:02Z"}}',
        "",
        ""
      ].join("\n")
    });
  });

  await page.goto("/");

  await expect(page.getByRole("button", { name: "Playwright chat" })).toBeVisible();
  await page.getByRole("textbox", { name: /message/i }).fill("Write a haiku");
  await page.getByRole("button", { name: /send message/i }).click();

  await expect(page.getByText("Write a haiku")).toBeVisible();
  await expect(page.getByText("Mocked assistant reply")).toBeVisible();
});
