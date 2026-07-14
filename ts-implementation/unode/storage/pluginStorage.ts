export type PluginStorage = {
  get: <T>(key: string) => Promise<T | null>;
  set: <T>(key: string, value: T) => Promise<void>;
  delete: (key: string) => Promise<void>;
};

const storageByPlugin = new Map<string, Map<string, unknown>>();

export function createPluginStorage(pluginId: string): PluginStorage {
  if (!storageByPlugin.has(pluginId)) {
    storageByPlugin.set(pluginId, new Map());
  }
  const bucket = storageByPlugin.get(pluginId) as Map<string, unknown>;

  return {
    async get<T>(key: string) {
      if (!bucket.has(key)) return null;
      return bucket.get(key) as T;
    },
    async set<T>(key: string, value: T) {
      bucket.set(key, value);
    },
    async delete(key: string) {
      bucket.delete(key);
    }
  };
}
