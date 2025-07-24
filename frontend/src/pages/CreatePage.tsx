import useRequest from '../hooks/useRequest';
import { useState } from 'react';
import { useTokenOrRedirect } from '../utils/jwt';

const CreatePage = () => {
  const token = useTokenOrRedirect();

  const [submitted, setSubmitted] = useState(false);
  const [prompt, setPrompt] = useState('');

  const { loading, error, sendRequest } = useRequest<null, { body: string }>(
    'POST',
    'prompts',
    null,
  );

  const handleReset = (e: React.FormEvent) => {
    e.preventDefault();
    setPrompt('');
    setSubmitted(false);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    void sendRequest({ token, body: { body: prompt } });
  };

  return (
    <>
      <h2>Create</h2>
      <hr />
      <form onSubmit={handleSubmit} onReset={handleReset}>
        <label>
          <h3>New prompt</h3>
          <input
            value={prompt}
            onChange={e => {
              setPrompt(e.target.value);
            }}
            placeholder='Tell me about a time...'
            disabled={submitted}
            required
            autoFocus
          />
        </label>
        <button type='submit' disabled={submitted}>
          Submit
        </button>
        <button type='reset'>Reset</button>
      </form>
      {submitted && (
        <>
          {loading && <p>Loading...</p>}
          {error && <p>Error: {error}</p>}
          {!loading && !error && <p>Successfully created!</p>}
        </>
      )}
      <hr />
      <h3>New post</h3>
      <p>Find a friend's prompt to respond to!</p>
    </>
  );
};

export default CreatePage;
