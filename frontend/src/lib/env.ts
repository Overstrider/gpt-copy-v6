export function getApiBaseUrl() {
  const value = process.env.NEXT_PUBLIC_API_BASE_URL?.trim();

  if (!value) {
    throw new Error("NEXT_PUBLIC_API_BASE_URL is not configured.");
  }

  return value.replace(/\/+$/, "");
}
