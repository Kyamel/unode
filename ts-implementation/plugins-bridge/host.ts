import type { HostApiBase } from '$lib/unode/api/host';
import type { CatalogApi, UsersApi, AuthApi, ReaderApi } from './capabilities';

export type MugenCatalogApi = CatalogApi;
export type MugenUsersApi = UsersApi;
export type MugenAuthApi = AuthApi;
export type MugenReaderApi = ReaderApi;

export type MugenHostApi = HostApiBase & {
  catalog: MugenCatalogApi;
  users: MugenUsersApi;
  auth: MugenAuthApi;
  reader: MugenReaderApi;
};
