import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import useRequest from '@/hooks/useRequest';
import { UsernamesResponseSchema, type UsernamesResponse } from '@/types';
import FriendRequest from '@/pages/Friends/Requests/FriendRequest';

const FriendRequestsPage = () => {
  const token = useTokenOrRedirect();

  const { data, error, loading, sendRequest } = useRequest<null, UsernamesResponse>({
    method: 'GET',
    endpoint: 'friends/requests',
    respSchema: UsernamesResponseSchema,
  });

  useEffect(() => {
    if (token) void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      <Link to='/friends'>
        <button type='button'>Back</button>
      </Link>
      <h2>Pending friend requests to you</h2>
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
