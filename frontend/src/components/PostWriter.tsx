import { useState } from 'react';
import TextareaAutosize from 'react-textarea-autosize';
import styles from '@/styles/shared.module.css';
import type { Prompt } from '@/types';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import useRequest from '@/hooks/useRequest';

const PostWriter = ({
  prompt,
  setRespondingToPrompt,
}: {
  prompt: Prompt;
  setRespondingToPrompt: (prompt: Prompt | null) => void;
}) => {
  const token = useTokenOrRedirect();
  const [postBody, setPostBody] = useState('');
  const { success, error, loading, sendRequest } = useRequest<
    { promptId: number; body: string },
    null
  >({ method: 'POST', endpoint: 'posts', respSchema: null });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (token) void sendRequest({ token, body: { promptId: prompt.id, body: postBody } });
  };

  return (
    <>
      <button
        type='button'
        onClick={() => {
          setRespondingToPrompt(null);
        }}
      >
        Back
      </button>
      <div style={{ textAlign: 'center' }}>
        <h2>New Post</h2>
        <p>
          <i>
            in response to {prompt.authorUsername}: "{prompt.body}"
          </i>
        </p>
        <hr />
        <form onSubmit={handleSubmit}>
          <TextareaAutosize
            className={styles.postBodyArea}
            value={postBody}
            onChange={e => {
              setPostBody(e.target.value);
            }}
            placeholder='Your thoughts here...'
            disabled={loading}
            required
            autoFocus
            minRows={5}
          />
          <br />
          <button type='submit' disabled={loading}>
            Post
          </button>
        </form>
        {loading && <p>Loading...</p>}
        {error && <p>Error: {error}</p>}
        {success && <p>Successfully created!</p>}
      </div>
    </>
  );
};

export default PostWriter;
