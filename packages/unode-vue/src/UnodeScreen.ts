// The Vue mount target: hosts the DOM produced by the universal renderer and
// wires the host-component portal. No Vue-specific renderer to maintain.

import { defineComponent, h as vueH, onMounted, onUnmounted, type PropType, ref } from "vue";
import {
  type OnAction,
  type Renderer,
  type RendererHandle,
  ScreenStore,
  defineRenderer,
} from "unode-web-renderer";

import { type HostComponents, VuePortalAdapter } from "./renderer";

const defaultRenderer = defineRenderer().build();

/**
 * Mounts a Unode screen. Pass a `renderer` to customize recipes and
 * `components` to back any `hostSlot` with native Vue components.
 */
export const UnodeScreen = defineComponent({
  name: "UnodeScreen",
  props: {
    store: { type: Object as PropType<ScreenStore>, required: true },
    onAction: { type: Function as PropType<OnAction>, required: false, default: undefined },
    renderer: { type: Object as PropType<Renderer>, required: false, default: undefined },
    components: { type: Object as PropType<HostComponents>, required: false, default: undefined },
  },
  setup(props) {
    const host = ref<HTMLElement | null>(null);
    let handle: RendererHandle | undefined;

    onMounted(() => {
      if (!host.value) return;
      const adapter = new VuePortalAdapter(props.components ?? {});
      handle = (props.renderer ?? defaultRenderer).mount(host.value, props.store, {
        onAction: props.onAction,
        portal: adapter,
      });
    });
    onUnmounted(() => handle?.unmount());

    return () => vueH("div", { ref: host, class: "unode-root" });
  },
});
