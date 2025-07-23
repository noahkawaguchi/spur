import { useEffect } from 'react';
import z from 'zod';
import { getToken } from '../utils/jwt';
import { Link, useNavigate } from 'react-router-dom';
import useRequest from '../hooks/useRequest';

const UsernamesResponseSchema = z.object({ usernames: z.array(z.string()) });
type UsernamesResponse = z.infer<typeof UsernamesResponseSchema>;

const FriendsPage = () => {
  const navigate = useNavigate();

  const {
    data: friendsData,
    error: friendsError,
    loading: friendsLoading,
    sendRequest: friendsSendRequest,
  } = useRequest<UsernamesResponse>('GET', 'friends', UsernamesResponseSchema);

  const {
    data: requestsData,
    error: requestsError,
    loading: requestsLoading,
    sendRequest: requestsSendRequest,
  } = useRequest<UsernamesResponse>('GET', 'friends/requests', UsernamesResponseSchema);

  useEffect(() => {
    const token = getToken();
    if (token) {
      void friendsSendRequest({ token });
      void requestsSendRequest({ token });
    } else {
      void navigate('/login');
    }
  }, [friendsSendRequest, requestsSendRequest, navigate]);

  return (
    <>
      <h2>Friends</h2>
      <hr />
      {friendsLoading && <p>Loading...</p>}
      {friendsError && <p>Error: {friendsError}</p>}
      {friendsData &&
        (friendsData.usernames.length ? (
          <>
            {friendsData.usernames.map(username => (
              <div key={username}>
                {username}{' '}
                <Link to={`/friends/${username}`}>
                  <button type='button'>View profile</button>
                </Link>{' '}
              </div>
            ))}
          </>
        ) : (
          <p>(No friends)</p>
        ))}

      <h3>Requests</h3>
      {requestsLoading && <p>Loading...</p>}
      {requestsError && <p>Error: {requestsError}</p>}
      {requestsData &&
        (requestsData.usernames.length ? (
          <p>Pending requests to you: {requestsData.usernames}</p>
        ) : (
          <p>(No pending requests)</p>
        ))}
    </>
  );
};

export default FriendsPage;
