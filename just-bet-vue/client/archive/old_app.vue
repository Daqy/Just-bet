<template>
  <div class="container">
    <AppLogo title="Just bet" />
    <RouterView v-slot="{ Component }">
      <suspense timeout="0">
        <template #default>
          <div>
            <AppCard :heading="heading">
              <template #icon>
                <component :is="Icons[heading.toLowerCase()]" />
              </template>
              <component :is="Component" :key="$route.path" />
            </AppCard>
            <footer v-if="authStore.token">
              <AppFooter />
            </footer>
          </div>
        </template>
        <template #fallback>
          <AppSkeletonCard />
        </template>
      </suspense>
    </RouterView>
  </div>
</template>
