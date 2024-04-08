import { fetch as f } from '@tauri-apps/plugin-http';

export function url(path: string) {
  return `http://localhost:3000${path}`;
}

export async function fetch(path: string, init?: RequestInit) {
  const response = await f(url(path), init);
  return response.blob();
}

export async function getBookCover(bookId: number) {
  return fetch(`/library/${bookId}/cover`);
}

export async function getBookPage(readerId: number, pageId: number) {
  return fetch(`/reader/${readerId}/${pageId}`);
}
