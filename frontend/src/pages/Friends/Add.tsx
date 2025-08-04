import { useState } from 'react';
import { Link } from 'react-router-dom';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import useRequest from '@/hooks/useRequest';
import { SuccessResponseSchema, type AddFriendRequest, type SuccessResponse } from '@/types';

const AddFriendPage = () => {
  const token = useTokenOrRedirect();
  const [username, setUsername] = useState('');
  const { data, error, loading, sendRequest } = useRequest<AddFriendRequest, SuccessResponse>({
    method: 'POST',
    endpoint: 'friends',
    respSchema: SuccessResponseSchema,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (token) void sendRequest({ token, body: { recipientUsername: username } });
    setUsername('');
  };

  return (
    <>
      <Link to='/friends'>
        <button type='button'>Back</button>
      </Link>
      <h2>Add a new friend</h2>
      <hr />
      <form onSubmit={handleSubmit}>
        <label>
          Username:{' '}
          <input
            value={username}
            onChange={e => {
              setUsername(e.target.value);
            }}
            placeholder='potential_friend123'
            disabled={loading}
            required
            autoFocus
          />
        </label>
        <button type='submit' disabled={loading}>
          Submit
        </button>
      </form>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && <p>{data.message}</p>}
    </>
  );
};

export default AddFriendPage;
