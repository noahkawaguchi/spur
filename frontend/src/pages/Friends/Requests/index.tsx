import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import useRequest from '@/hooks/useRequest';
import FriendRequest from '@/pages/Friends/Requests/FriendRequest';
import { StringArraySchema } from '@/types';

const FriendRequestsPage = () => {
  const token = useTokenOrRedirect();

  const {
    data: usernames,
    error,
    loading,
    sendRequest,
  } = useRequest<null, string[]>({
    method: 'GET',
    endpoint: 'friends/requests',
    respSchema: StringArraySchema,
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
      {usernames &&
        (usernames.length ? (
          <table>
            <tbody>
              {usernames.map(username => (
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
