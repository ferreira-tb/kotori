const contentHeight = Symbol('contentHeight') as ComputedSymbol<number>;
const windowHeight = Symbol('windowHeight') as RefSymbol<number>;

export const symbols = {
  contentHeight,
  windowHeight
} as const;
