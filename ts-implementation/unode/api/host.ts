import type { ResolvedRouteInfo } from './contracts';

export type PaginatedResult<T> = {
  data: T[];
  lastCursor?: string | null;
  meta?: Record<string, unknown>;
};

export type MessageValue = string | number | boolean | null | undefined;
export type MessageValues = Record<string, MessageValue>;
export interface MessageCatalog {
  [key: string]: string | MessageCatalog;
}
export type MessageCatalogs = Record<string, MessageCatalog>;

export type Translator = {
  t: (key: string, values?: MessageValues, fallback?: string) => string;
  locale: () => string;
};

export type SessionInfo = {
  userId: string;
  expiresAt?: string | null;
};

export type HostApiBase = {
  navigation: NavigationApi;
  feedback: FeedbackApi;
  storage: HostStorageApi;
  http: HttpApi;
  i18n: HostI18nApi;
  events: HostEventsApi;
  system: SystemApi;
};

export type HostApi = HostApiBase;

export type NavigationApi = {
  navigate: (to: string, options?: { replace?: boolean; state?: unknown }) => Promise<void>;
  openExternal: (url: string, options?: { target?: '_blank' | '_self' }) => Promise<void>;
  openScreen: (input: {
    screenKind: string;
    params?: Record<string, string>;
    query?: Record<string, string>;
  }) => Promise<void>;
  getCurrentRoute: () => Promise<ResolvedRouteInfo>;
};

export type FeedbackApi = {
  toast: (input: {
    title: string;
    message?: string;
    tone?: 'info' | 'success' | 'warning' | 'danger';
  }) => void;
  confirm: (input: {
    title: string;
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
  }) => Promise<boolean>;
};

export type HostStorageApi = {
  getScoped: <T>(scope: string, key: string) => Promise<T | null>;
  setScoped: <T>(scope: string, key: string, value: T) => Promise<void>;
};

export type HttpApi = {
  request: <T = unknown>(input: {
    method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
    url: string;
    headers?: Record<string, string>;
    query?: Record<string, string | number | boolean>;
    body?: unknown;
  }) => Promise<HttpResponse<T>>;
};

export type HttpResponse<T> = {
  status: number;
  data: T;
  headers?: Record<string, string>;
};

export type HostI18nApi = {
  getLocale: () => string;
  translate: (input: {
    catalogs: MessageCatalogs;
    key: string;
    values?: MessageValues;
    fallback?: string;
  }) => string;
  getTranslator: (catalogs: MessageCatalogs) => Translator;
};

export type HostEventBase = { type: 'route.changed'; route: ResolvedRouteInfo };

export type ScreenRefreshEvent = {
  type: 'screen.refresh';
  screenKind?: string;
  pathname?: string;
};

export type HostEvent = HostEventBase | ScreenRefreshEvent | { type: string; [key: string]: unknown };

export type HostEventsApi = {
  emit: (event: HostEvent) => void;
  on: <T extends HostEvent['type']>(
    type: T,
    handler: (event: Extract<HostEvent, { type: T }>) => void | Promise<void>
  ) => () => void;
};

export type SystemApi = {
  getRuntimeInfo: () => Promise<{
    platform: 'web' | 'desktop' | 'tui';
    appVersion: string;
    pluginApiVersion: string;
  }>;
};
