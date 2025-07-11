<script setup lang="ts">
import AppHeader from './components/navigation/AppHeader.vue'
</script>

<template>
  <div :class="$style.tab">
    <AppHeader />
  </div>
  <div :class="$style.container">
    <router-view />
  </div>
</template>

<style module>
.tab {
  top: 0;
  width: 100%;
  min-height: clamp(60px, 10vh, 100px);
  place-items: center;
}

.container {
  display: flex;
  place-items: center;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  overflow-y: auto;
  min-height: calc(100vh - clamp(60px, 10vh, 100px));
  width: 100%;
}
</style>
