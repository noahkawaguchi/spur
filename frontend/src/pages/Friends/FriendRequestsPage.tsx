import { useEffect } from 'react';
import useRequest from '../../hooks/useRequest';
import { UsernamesResponseSchema, type UsernamesResponse } from '../../types';
import { useTokenOrRedirect } from '../../utils/jwt';
import FriendRequest from './FriendRequest';

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
      <h2>Pending friend requests</h2>
      <hr />
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data &&
        (data.usernames.length ? (
          <table>
            <tbody>
              {data.usernames.map(username => (
                <FriendRequest key={username} username={username} />
              ))}
            </tbody>
          </table>
        ) : (
          <p>(No pending friend requests)</p>
        ))}
    </>
  );
};

export default FriendRequestsPage;
