import { symbols } from 'manatsu';
import type { Router } from 'vue-router';

export function registerNavigationGuards(router: Router) {
  router.beforeEach((to, from) => {
    if (to.name === 'reader') {
      const showSidebar = inject(symbols.showScaffoldSidebar);
      if (showSidebar) showSidebar.value = false;
    } else if (from.name === 'reader') {
      const showSidebar = inject(symbols.showScaffoldSidebar);
      if (showSidebar) showSidebar.value = true;
    }
  });
}
