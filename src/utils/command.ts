import { defineInvoke } from 'manatsu';

export enum Command {
  OpenWithDialog = 'open_with_dialog',
  Version = 'version'
}

export const useInvoke = defineInvoke(Command);
