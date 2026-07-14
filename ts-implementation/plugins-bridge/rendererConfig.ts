import type { RendererConfig } from '$lib/unode/renderer/config';
import { tuiRendererConfig, webRendererConfig } from '$lib/unode/renderer/config';

export type RendererProfile = 'web' | 'tui';

let rendererProfile: RendererProfile = 'web';

export function setRendererProfile(profile: RendererProfile) {
  rendererProfile = profile;
}

export function getRendererProfile(): RendererProfile {
  return rendererProfile;
}

const baseConfig = getRendererProfile() === 'tui' ? tuiRendererConfig : webRendererConfig;

export function createAppRendererConfig(
  override?: Partial<RendererConfig>
): RendererConfig {
  return {
    breakpoints: {
      ...baseConfig.breakpoints,
      ...(override?.breakpoints ?? {})
    },
    collections: {
      ...baseConfig.collections,
      ...(override?.collections ?? {})
    },
    navigation: {
      grid: {
        ...baseConfig.navigation.grid,
        ...(override?.navigation?.grid ?? {})
      },
      list: {
        ...baseConfig.navigation.list,
        ...(override?.navigation?.list ?? {})
      },
      tabs: {
        ...baseConfig.navigation.tabs,
        ...(override?.navigation?.tabs ?? {})
      }
    }
  };
}

export const appRendererConfig = createAppRendererConfig();
