import { useEffect } from 'react';
import { useTokenOrRedirect } from '../../utils/jwt';
import { Link, useNavigate } from 'react-router-dom';
import useRequest from '../../hooks/useRequest';
import { UsernamesResponseSchema, type UsernamesResponse } from '../../types';

const FriendsPage = () => {
  const token = useTokenOrRedirect();
  const navigate = useNavigate();

  const { data, error, loading, sendRequest } = useRequest<UsernamesResponse>(
    'GET',
    'friends',
    UsernamesResponseSchema,
  );

  useEffect(() => {
    void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      <h2>Friends</h2>
      <button type='button' onClick={() => void navigate('/friends/requests')}>
        See pending requests
      </button>
      <button type='button' onClick={() => void navigate('/friends/add')}>
        Add a new friend
      </button>
      <hr />
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data &&
        (data.usernames.length ? (
          <>
            <h3>Current friends</h3>
            {data.usernames.map(username => (
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
    </>
  );
};

export default FriendsPage;
