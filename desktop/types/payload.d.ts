type BookAddedPayload = Readonly<LibraryBook>;

interface BookRemovedPayload {
  readonly id: number;
}

interface CoverExtractedPayload {
  readonly id: number;
  readonly path: string;
}

interface PageDeletedPayload {
  readonly name: string;
}

interface RatingUpdatedPayload {
  readonly id: number;
  readonly rating: number;
}

interface ReadUpdatedPayload {
  readonly id: number;
  readonly read: boolean;
}
