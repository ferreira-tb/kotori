import { defineInvoke } from 'manatsu';

export enum Command {
  OpenFile = 'open_file',
  Version = 'version'
}

export const useInvoke = defineInvoke(Command);
