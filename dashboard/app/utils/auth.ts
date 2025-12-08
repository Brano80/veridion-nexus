// Utility functions for authentication

export function getAuthToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("token");
}

export function setAuthToken(token: string): void {
  if (typeof window === "undefined") return;
  localStorage.setItem("token", token);
}

export function removeAuthToken(): void {
  if (typeof window === "undefined") return;
  localStorage.removeItem("token");
}

export function getAuthHeaders(): HeadersInit {
  const token = getAuthToken();
  return {
    "Authorization": `Bearer ${token || ""}`,
    "Content-Type": "application/json",
  };
}

