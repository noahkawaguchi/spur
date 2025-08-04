import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { useState } from 'react';

const CreatePage = () => {
  const token = useTokenOrRedirect();
  const [prompt, setPrompt] = useState('');
  const { success, loading, error, sendRequest } = useRequest<{ body: string }, null>({
    method: 'POST',
    endpoint: 'prompts',
    respSchema: null,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (token) void sendRequest({ token, body: { body: prompt } });
  };

  return (
    <>
      <h2>Create</h2>
      <hr />
      <form onSubmit={handleSubmit}>
        <label>
          <h3>New prompt</h3>
          <input
            value={prompt}
            onChange={e => {
              setPrompt(e.target.value);
            }}
            placeholder='Tell me about a time...'
            disabled={loading}
            required
            autoFocus
            style={{ width: '95%' }}
          />
        </label>
        <br />
        <button type='submit' disabled={loading}>
          Submit
        </button>
      </form>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {success && <p>Successfully created!</p>}
      <hr />
      <h3>New post</h3>
      <p>Find a friend's prompt to respond to!</p>
    </>
  );
};

export default CreatePage;
