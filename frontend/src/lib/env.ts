export function getApiBaseUrl() {
  const value = process.env.NEXT_PUBLIC_API_BASE_URL?.trim();

  if (!value) {
    return "http://localhost:8080";
  }

  return value.replace(/\/+$/, "");
}
