import type { PluginManifest, PermissionRequest } from '../core/runtime';

export function hasPermission(plugin: PluginManifest, permission: string): boolean {
  return plugin.permissions?.some((entry: PermissionRequest<string>) => entry.permission === permission) ?? false;
}

export function guardPermission<T extends (...args: never[]) => unknown>(
  plugin: PluginManifest,
  permission: string,
  fn: T
): T {
  return (((...args: unknown[]) => {
    if (!hasPermission(plugin, permission)) {
      throw new Error(`Missing permission: ${permission}`);
    }
    return fn(...(args as Parameters<T>));
  }) as unknown) as T;
}
