import type { PaginatedResult, SessionInfo } from '$lib/unode/api/host';
import type {
  ChapterPagesResult,
  WorkSummary,
  WorkDetails,
  ChapterSummary,
  ChapterDetails,
  WorkStaff,
  WorkRelation,
  CurrentUser,
  UserSummary
} from './models';

export type CatalogApi = {
  listWorks: (input?: {
    limit?: number;
    cursor?: string | null;
    filters?: {
      title?: string;
      workType?: string | string[];
      status?: string | string[];
      format?: string;
      publishedAfter?: string;
    };
  }) => Promise<PaginatedResult<WorkSummary>>;
  searchWorks: (input: {
    query: string;
    page?: number;
    filters?: Record<string, unknown>;
  }) => Promise<PaginatedResult<WorkSummary>>;
  getWorkById: (id: string) => Promise<WorkDetails | null>;
  getChapterById: (id: string) => Promise<ChapterDetails | null>;
  getChapterPages: (chapterId: string) => Promise<ChapterPagesResult>;
  listChaptersByWork: (workId: string) => Promise<ChapterSummary[]>;
  listWorkStaff: (workId: string) => Promise<WorkStaff[]>;
  listWorkRelations: (workId: string) => Promise<WorkRelation[]>;
};

export type UsersApi = {
  getCurrentUser: () => Promise<CurrentUser | null>;
  getUserById: (id: string) => Promise<UserSummary | null>;
  isLoggedIn: () => Promise<boolean>;
};

export type AuthApi = {
  requireLogin: (options?: { reason?: string }) => Promise<boolean>;
  getSessionInfo: () => Promise<SessionInfo | null>;
};

export type ReaderApi = {
  openChapter: (chapterId: string) => Promise<void>;
  setProgress: (input: { chapterId: string; page: number }) => Promise<void>;
  openImages: (images: string[], startIndex?: number, key?: string) => Promise<void>;
};
