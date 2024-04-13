type BookAddedPayload = LibraryBook;

interface BookRemovedPayload {
  readonly id: number;
}

interface CoverExtractedPayload {
  readonly id: number;
  readonly path: string;
}

interface DeletePageRequestedPayload {
  readonly page: number;
}

interface RatingUpdatedPayload {
  readonly id: number;
  readonly rating: number;
}

interface RemoveBookRequestedPayload {
  readonly id: number;
  readonly title: string;
}
