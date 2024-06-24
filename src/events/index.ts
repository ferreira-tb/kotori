import { useConfigStore } from '@/stores/config';

enum Event {
  ConfigUpdated = 'config_updated'
}

export function setupGlobalEventListeners() {
  onConfigUpdated().catch(handleError);
}

function onConfigUpdated() {
  return listen(Event.ConfigUpdated, () => {
    const config = useConfigStore();
    config.load().catch(handleError);
  });
}
