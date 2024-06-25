import { getServerPort } from './commands';
import { fetch as f } from '@tauri-apps/plugin-http';

interface Server {
  port?: number;
}

const server: Server = {};

async function toUrl(path: string) {
  server.port ||= await getServerPort();
  return `http://127.0.0.1:${server.port}/kotori/${path}`;
}

async function fetch(path: string, init?: RequestInit) {
  const url = await toUrl(path);
  const response = await f(url, init);
  return response.blob();
}

export async function getBookCover(bookId: number) {
  return fetch(`library/${bookId}/cover`);
}

export async function getBookPage(windowId: number, pageId: number) {
  return fetch(`reader/${windowId}/${pageId}`);
}
