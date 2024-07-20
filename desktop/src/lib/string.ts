export function contains(search: string, ...str: string[]): boolean {
  const lowercase = search.toLocaleLowerCase();
  return str.some((s) => s.toLocaleLowerCase().includes(lowercase));
}
