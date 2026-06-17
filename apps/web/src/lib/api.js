const DEFAULT_API = 'http://127.0.0.1:8080';

export const API_URL = (import.meta.env.VITE_API_URL || DEFAULT_API).replace(/\/+$/, '');

export async function apiFetch(endpoint, body, options = {}) {
  let res;
  try {
    res = await fetch(API_URL + endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      signal: options.signal,
    });
  } catch {
    // Return a readable message instead of throwing so the UI can surface a
    // helpful "start the API" hint in the error banner.
    return { ok: false, error: `Cannot reach API at ${API_URL}\nStart: cargo run -p lod-api` };
  }

  const text = await res.text();
  if (!text) return { ok: false, error: `Empty response (HTTP ${res.status})` };

  let data;
  try {
    data = JSON.parse(text);
  } catch {
    return { ok: false, error: `Bad JSON (HTTP ${res.status}): ${text.slice(0, 120)}` };
  }

  if (!res.ok) return { ok: false, error: data.error || `HTTP ${res.status}` };
  return { ok: true, data };
}
