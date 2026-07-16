<script setup lang="ts">
// A host-owned Vue component. The plugin never imports it — it asks for a
// semantic "Button" via `hostSlot`, and this app decides what it looks like.
import { computed } from "vue";
import type { OnAction } from "unode-vue";

const props = defineProps<{
  children?: unknown;
  intent?: unknown;
  action?: unknown;
  disabled?: unknown;
  dispatch: OnAction;
}>();

const intent = computed(() => {
  const value = props.intent;
  return value === "primary" || value === "secondary" || value === "ghost" || value === "danger"
    ? value
    : "secondary";
});
const isDisabled = computed(() => props.disabled === true);

function onClick() {
  if (!isDisabled.value && props.action) props.dispatch(props.action as { t: string });
}
</script>

<template>
  <button
    :class="`ds-button ds-button--${intent}`"
    :disabled="isDisabled"
    :data-intent="intent"
    @click="onClick"
  >
    {{ String(props.children ?? "") }}
  </button>
</template>
