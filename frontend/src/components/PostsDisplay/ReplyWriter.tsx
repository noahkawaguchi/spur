import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { useState } from 'react';
import styles from './PostsDisplay.module.css';
import TextareaAutosize from 'react-textarea-autosize';

const ReplyWriter = ({ parentId, cancelFn }: { parentId: number; cancelFn: () => void }) => {
  const token = useTokenOrRedirect();
  const [postBody, setPostBody] = useState('');

  const { success, error, loading, sendRequest } = useRequest<
    { parentId: number; body: string },
    null
  >({ method: 'POST', endpoint: 'posts', respSchema: null });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (token) void sendRequest({ token, body: { parentId, body: postBody } });
  };

  return (
    <>
      <hr />
      <form onSubmit={handleSubmit}>
        <label>
          New Reply:
          <TextareaAutosize
            className={styles.postBodyArea}
            value={postBody}
            onChange={e => setPostBody(e.target.value)}
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
        <button type='button' onClick={cancelFn}>
          Cancel
        </button>
      </form>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {success && <p>Successfully created!</p>}
    </>
  );
};

export default ReplyWriter;
