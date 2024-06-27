import { getServerPort } from './commands';
import { fetch as f } from '@tauri-apps/plugin-http';

interface Server {
  port?: number;
}

const server: Server = {};

async function toUrl(endpoint: string) {
  server.port ||= await getServerPort();
  return `http://127.0.0.1:${server.port}/kotori/${endpoint}`;
}

async function fetch(endpoint: string, init?: RequestInit) {
  const url = await toUrl(endpoint);
  const response = await f(url, init);
  return response.blob();
}

export async function getBookCover(bookId: number) {
  return fetch(`library/${bookId}/cover`);
}

export async function getBookPage(windowId: number, name: string) {
  return fetch(`reader/${windowId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name })
  });
}
