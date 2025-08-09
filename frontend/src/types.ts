import { z } from 'zod';

export const TokenResponseSchema = z.object({ token: z.string() });
export type TokenResponse = z.infer<typeof TokenResponseSchema>;

export const SuccessResponseSchema = z.object({ message: z.string() });
export type SuccessResponse = z.infer<typeof SuccessResponseSchema>;

export interface AddFriendRequest {
  recipientUsername: string;
}

export const PostSchema = z.object({
  id: z.number(),
  authorUsername: z.string(),
  parentId: z.nullable(z.number()),
  body: z.string(),
  createdAtMs: z.number(),
  editedAtMs: z.nullable(z.number()),
  archivedAtMs: z.nullable(z.number()),
  deletedAtMs: z.nullable(z.number()),
});
export type Post = z.infer<typeof PostSchema>;

export const PostsResponseSchema = z.object({ posts: z.array(PostSchema) });
export type PostsResponse = z.infer<typeof PostsResponseSchema>;

export const UsernamesResponseSchema = z.object({ usernames: z.array(z.string()) });
export type UsernamesResponse = z.infer<typeof UsernamesResponseSchema>;
