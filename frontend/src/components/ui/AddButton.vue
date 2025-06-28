<script setup lang="ts">
interface Props {
  disabled?: boolean;
  label?: string;
}

defineProps<Props>();
defineEmits<{
  click: [];
}>();
</script>

<template>
  <button :class="$style.addButton" @click="$emit('click')" :disabled="disabled" :aria-label="label">
    <span :class="$style.icon">+</span>
  </button>
</template>

<style module>
.addButton {
  position: fixed;
  bottom: 32px;
  right: 32px;
  width: 60px;
  height: 60px;
  background: var(--color-accent);
  color: var(--color-background);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 4px 16px color-mix(in srgb, var(--color-accent) 40%, transparent);
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.addButton:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-accent) 80%, black);
  transform: scale(1.1);
  box-shadow: 0 6px 20px color-mix(in srgb, var(--color-accent) 50%, transparent);
}

.addButton:active:not(:disabled) {
  transform: scale(0.95);
}

.addButton:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-accent) 20%, transparent);
}

.icon {
  font-size: 24px;
  font-weight: 300;
  line-height: 1;
}
</style>
