import type { Post } from '@/types';

export const dummyUsernames = ['jeff1', 'jeff2', 'jefe', 'jpeg'];

export const dummyPosts: Post[] = [
  {
    id: 245,
    authorUsername: 'jasmine',
    parentId: 99,
    body: 'hello from my post!',
    createdAtMs: 490_924_444,
    editedAtMs: null,
    archivedAtMs: null,
    deletedAtMs: null,
  },
  {
    id: 54_134,
    authorUsername: 'jazz-men',
    parentId: 99,
    body: 'cool post this is',
    createdAtMs: 924_242,
    editedAtMs: 2_242_009,
    archivedAtMs: 24_242_222,
    deletedAtMs: 25_999_002,
  },
  {
    id: 22,
    authorUsername: 'jessica_ica',
    parentId: 21,
    body: 'this is an old post',
    createdAtMs: 100,
    editedAtMs: null,
    archivedAtMs: 924,
    deletedAtMs: null,
  },
];
