import { z } from 'zod';

export const TokenResponseSchema = z.object({ token: z.string() });
export type TokenResponse = z.infer<typeof TokenResponseSchema>;

export const SuccessResponseSchema = z.object({ message: z.string() });
export type SuccessResponse = z.infer<typeof SuccessResponseSchema>;

export interface AddFriendRequest {
  recipientUsername: string;
}

export const StringArraySchema = z.array(z.string());

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
export const PostArraySchema = z.array(PostSchema);
