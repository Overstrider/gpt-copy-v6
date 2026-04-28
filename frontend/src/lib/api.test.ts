import { beforeEach, describe, expect, it, vi } from "vitest";

import { ApiValidationError, listConversations } from "./api";

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
});
