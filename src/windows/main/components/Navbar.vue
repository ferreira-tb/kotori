<script setup lang="ts">
import { RouteName } from '../router';
</script>

<template>
  <NavigationMenuRoot id="kt-navbar">
    <NavigationMenuList id="kt-navbar-list">
      <NavigationMenuItem>
        <NavigationMenuLink as-child>
          <RouterLink class="kt-navbar-link" :to="{ name: RouteName.Library }">
            <span>Library</span>
          </RouterLink>
        </NavigationMenuLink>
      </NavigationMenuItem>

      <NavigationMenuItem>
        <NavigationMenuLink as-child>
          <RouterLink class="kt-navbar-link" :to="{ name: RouteName.Collection }">
            <span>Collection</span>
          </RouterLink>
        </NavigationMenuLink>
      </NavigationMenuItem>

      <NavigationMenuItem>
        <NavigationMenuLink as-child>
          <RouterLink class="kt-navbar-link" :to="{ name: RouteName.BookTag }">
            <span>Tags</span>
          </RouterLink>
        </NavigationMenuLink>
      </NavigationMenuItem>
    </NavigationMenuList>
  </NavigationMenuRoot>
</template>
