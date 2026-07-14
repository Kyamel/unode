export type WorkTagPreview = {
  id: string;
  name: string;
  category?: string | null;
  provider?: string | null;
  external_id?: string | null;
  description?: string | null;
};

export type WorkCover = {
  url: string;
  locale?: string | null;
};

export type WorkSummary = {
  id: string;
  title: string;
  canonical_slug?: string | null;
  cover?: WorkCover | null;
  work_type?: string | null;
  format?: string | null;
  status?: string | null;
  year?: number | null;
};

export type WorkDetails = WorkSummary & {
  original_language?: string | null;
  publication_demographic?: string | null;
  content_rating?: string | null;
  tags?: WorkTagPreview[] | null;
  genres?: WorkTagPreview[] | null;
  themes?: WorkTagPreview[] | null;
  content?: WorkTagPreview[] | null;
  visibility?: string | null;
  source_kind?: string | null;
  is_locked?: boolean | null;
  updated_at?: string | null;
  created_at?: string | null;
};

export type WorkStaff = {
  id: string;
  name?: string | null;
  role?: string | null;
};

export type WorkRelation = {
  id?: string;
  relation_type?: string | null;
  related_work_id?: string | null;
  related_work_title?: string | null;
};

export type ChapterSummary = {
  id: string;
  title?: string | null;
  language_code?: string | null;
  number?: string | number | null;
  number_numeric?: number | null;
  volume?: number | null;
  sort_key?: string | null;
  source_url?: string | null;
};

export type ChapterDetails = ChapterSummary & {
  pages_count?: number | null;
  released_at?: string | null;
};

export type ChapterPage = {
  chapter_id: string;
  page_index: number;
  asset_id: string;
  created_at?: string | null;
  updated_at?: string | null;
};

export type ChapterReadingPayload = {
  id: string;
  chapter_id: string;
  provider: string;
  base_url?: string | null;
  hash?: string | null;
  data?: string[] | null;
  data_saver?: string[] | null;
  created_at?: string | null;
  updated_at?: string | null;
};

export type ChapterPagesResult = {
  pages?: ChapterPage[] | null;
  reading_payloads?: ChapterReadingPayload[] | null;
};

export type UserSummary = {
  id: string;
  username?: string | null;
  display_name?: string | null;
  avatar_url?: string | null;
};

export type CurrentUser = UserSummary;
