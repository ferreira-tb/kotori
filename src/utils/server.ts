const API_URL = 'http://localhost:3000';

export function api(path: string) {
  return `${API_URL}${path}`;
}
