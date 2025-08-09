import type { Post } from '@/types';
import styles from '@/styles/shared.module.css';
import { howLongAgo } from '@/utils/fmt';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { useState } from 'react';
import useRequest from '@/hooks/useRequest';
import TextareaAutosize from 'react-textarea-autosize';

// TODO:
//   - move styles to a module here instead of shared
//   - add buttons to get parent post and children posts

const SinglePostDisplay = ({
  readingPost,
  setReadingPost,
}: {
  readingPost: Post;
  setReadingPost: (readingPost: Post | null) => void;
}) => {
  const token = useTokenOrRedirect();
  const [postBody, setPostBody] = useState('');
  const [replying, setReplying] = useState(false);
  const { success, error, loading, sendRequest } = useRequest<
    { parentId: number; body: string },
    null
  >({ method: 'POST', endpoint: 'posts', respSchema: null });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (token) void sendRequest({ token, body: { parentId: readingPost.id, body: postBody } });
  };

  return (
    <>
      <button
        type='button'
        onClick={() => {
          setReadingPost(null);
        }}
      >
        Back
      </button>
      <div style={{ textAlign: 'center' }}>
        <h2>Post by {readingPost.authorUsername}</h2>
        <p>created {howLongAgo(readingPost.createdAtMs)} ago</p>
        <hr />
        <p className={styles.postBodyArea} style={{ whiteSpace: 'pre-line', textAlign: 'justify' }}>
          {readingPost.body}
        </p>
        <button
          type='button'
          onClick={() => {
            setReplying(true);
          }}
          hidden={replying}
        >
          Reply
        </button>
        {replying && (
          <>
            <hr />
            <form onSubmit={handleSubmit}>
              <label>
                New Reply:
                <TextareaAutosize
                  className={styles.postBodyArea}
                  value={postBody}
                  onChange={e => {
                    setPostBody(e.target.value);
                  }}
                  placeholder='Your thoughts...'
                  disabled={loading}
                  required
                  autoFocus
                  minRows={5}
                />
              </label>
              <br />
              <button type='submit' disabled={loading}>
                Post
              </button>
              <button
                type='button'
                onClick={() => {
                  setReplying(false);
                }}
              >
                Cancel
              </button>
            </form>
            {loading && <p>Loading...</p>}
            {error && <p>Error: {error}</p>}
            {success && <p>Successfully created!</p>}
          </>
        )}
      </div>
    </>
  );
};

export default SinglePostDisplay;
