import { useState } from 'react';
import type { Prompt } from '../../types';
import useRequest from '../../hooks/useRequest';
import { useTokenOrRedirect } from '../../utils/jwt';
import TextareaAutosize from 'react-textarea-autosize';
import styles from './PostReaderWriter.module.css';

const PostWriter = ({
  prompt,
  setRespondingToPrompt: setWritingPost,
}: {
  prompt: Prompt;
  setRespondingToPrompt: (prompt: Prompt | null) => void;
}) => {
  const token = useTokenOrRedirect();
  const [postBody, setPostBody] = useState('');
  const { success, error, loading, sendRequest } = useRequest<
    null,
    { promptId: number; body: string }
  >('POST', 'posts', null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void sendRequest({ token, body: { promptId: prompt.id, body: postBody } });
  };

  return (
    <>
      <button
        type='button'
        onClick={() => {
          setWritingPost(null);
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
            className={styles.postWritingArea}
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
