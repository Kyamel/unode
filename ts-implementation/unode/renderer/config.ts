export type RendererBreakpoints = {
  sm: number;
  md: number;
  lg: number;
  xl: number;
};

export type RendererZone = 'header' | 'sidebar' | 'main';
export type RendererContainerAxis = 'vertical' | 'horizontal' | 'both';
export type RendererGridStrategy = 'cssCols' | 'spatial';

export type NavigationDefaults = {
  grid: {
    zone: RendererZone;
    axis: RendererContainerAxis;
    strategy: RendererGridStrategy;
    pageRows: number;
  };
  list: {
    zone: RendererZone;
    axis: RendererContainerAxis;
    wrap: boolean;
    pageJump: number;
  };
  tabs: {
    zone: RendererZone;
    axis: RendererContainerAxis;
    wrap: boolean;
  };
};

export type RendererConfig = {
  breakpoints: RendererBreakpoints;
  collections: {
    autoLoadContinuation: boolean;
  };
  navigation: NavigationDefaults;
};

export const webRendererConfig: RendererConfig = {
  breakpoints: {
    sm: 640,
    md: 768,
    lg: 1024,
    xl: 1280
  },
  collections: {
    autoLoadContinuation: true
  },
  navigation: {
    grid: {
      zone: 'main',
      axis: 'both',
      strategy: 'cssCols',
      pageRows: 2
    },
    list: {
      zone: 'main',
      axis: 'vertical',
      wrap: false,
      pageJump: 2
    },
    tabs: {
      zone: 'main',
      axis: 'horizontal',
      wrap: true
    }
  }
};

export const tuiRendererConfig: RendererConfig = {
  breakpoints: {
    sm: 80,
    md: 120,
    lg: 160,
    xl: 200
  },
  collections: {
    autoLoadContinuation: false
  },
  navigation: {
    grid: {
      zone: 'main',
      axis: 'both',
      strategy: 'cssCols',
      pageRows: 2
    },
    list: {
      zone: 'main',
      axis: 'vertical',
      wrap: false,
      pageJump: 2
    },
    tabs: {
      zone: 'main',
      axis: 'horizontal',
      wrap: true
    }
  }
};

export const defaultRendererConfig = webRendererConfig;

export function createRendererConfig(override?: Partial<RendererConfig>): RendererConfig {
  return {
    breakpoints: {
      ...defaultRendererConfig.breakpoints,
      ...(override?.breakpoints ?? {})
    },
    collections: {
      ...defaultRendererConfig.collections,
      ...(override?.collections ?? {})
    },
    navigation: {
      grid: {
        ...defaultRendererConfig.navigation.grid,
        ...(override?.navigation?.grid ?? {})
      },
      list: {
        ...defaultRendererConfig.navigation.list,
        ...(override?.navigation?.list ?? {})
      },
      tabs: {
        ...defaultRendererConfig.navigation.tabs,
        ...(override?.navigation?.tabs ?? {})
      }
    }
  };
}
