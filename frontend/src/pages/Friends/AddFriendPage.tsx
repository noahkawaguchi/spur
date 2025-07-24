import { useState } from 'react';
import useRequest from '../../hooks/useRequest';
import { type AddFriendRequest, type SuccessResponse, SuccessResponseSchema } from '../../types';
import { useTokenOrRedirect } from '../../utils/jwt';
import { Link } from 'react-router-dom';

const AddFriendPage = () => {
  const token = useTokenOrRedirect();

  const [username, setUsername] = useState('');

  const { data, error, loading, sendRequest } = useRequest<SuccessResponse, AddFriendRequest>(
    'POST',
    'friends',
    SuccessResponseSchema,
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void sendRequest({ token, body: { recipientUsername: username } });
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
            placeholder='alice123'
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
