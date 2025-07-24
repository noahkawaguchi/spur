import { z } from 'zod';

export const SuccessResponseSchema = z.object({ message: z.string() });
export type SuccessResponse = z.infer<typeof SuccessResponseSchema>;

export interface AddFriendRequest {
  recipientUsername: string;
}

const PromptSchema = z.object({ id: z.number(), authorUsername: z.string(), body: z.string() });

const PostSchema = z.object({
  id: z.number(),
  authorUsername: z.string(),
  prompt: PromptSchema,
  body: z.string(),
});
export type Post = z.infer<typeof PostSchema>;

export const ContentSchema = z.object({
  prompts: z.array(PromptSchema),
  posts: z.array(PostSchema),
});
export type Content = z.infer<typeof ContentSchema>;

export const UsernamesResponseSchema = z.object({ usernames: z.array(z.string()) });
export type UsernamesResponse = z.infer<typeof UsernamesResponseSchema>;
