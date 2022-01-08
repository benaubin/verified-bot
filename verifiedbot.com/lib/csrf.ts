let csrf_token: string | undefined;
export const getCsrfToken = () => {
  if (typeof window == "undefined") throw new Error("must be called in browser context");
  if (csrf_token == null) {
    csrf_token = document.cookie
      .trim()
      .split(";")
      .map((cookie) => cookie.trim().split("=", 2))
      .find(([k]) => k == "csrf_token")?.[1];
  }
  if (csrf_token == null) {
    csrf_token = crypto.randomUUID!();
    document.cookie = "csrf_token=" + csrf_token;
  } 
  return csrf_token;
};
