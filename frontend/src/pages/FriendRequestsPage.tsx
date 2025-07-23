import { useEffect } from 'react';
import useRequest from '../hooks/useRequest';
import { UsernamesResponseSchema, type UsernamesResponse } from '../types';
import { useTokenOrRedirect } from '../utils/jwt';

const FriendRequestsPage = () => {
  const token = useTokenOrRedirect();

  const { data, error, loading, sendRequest } = useRequest<UsernamesResponse>(
    'GET',
    'friends/requests',
    UsernamesResponseSchema,
  );

  useEffect(() => {
    void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      <h2>Friend requests</h2>
      <hr />
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data &&
        (data.usernames.length ? (
          data.usernames.map(username => (
            <div key={username}>
              <p>{username}</p>
            </div>
          ))
        ) : (
          <p>(No pending friend requests)</p>
        ))}
    </>
  );
};

export default FriendRequestsPage;
