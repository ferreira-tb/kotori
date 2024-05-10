type BookAddedPayload = LibraryBook;

interface BookRemovedPayload {
  readonly id: number;
}

interface CoverExtractedPayload {
  readonly id: number;
  readonly path: string;
}

interface RatingUpdatedPayload {
  readonly id: number;
  readonly rating: number;
}
