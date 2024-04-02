export class Page {
  public readonly id: number;
  public status: BookStatus = 'not started';
  public url: string | null = null;

  private constructor(id: number) {
    this.id = id;
  }

  public static from(page: number): Page;
  public static from(pages: number[]): Page[];
  public static from(source: number | number[]): Page | Page[] {
    if (Array.isArray(source)) {
      return source.map((id) => new Page(id));
    }

    return new Page(source);
  }
}
