import { defineInvoke } from 'manatsu';

export enum Command {
  OpenBook = 'open_book',
  Version = 'version'
}

export const useInvoke = defineInvoke(Command);
