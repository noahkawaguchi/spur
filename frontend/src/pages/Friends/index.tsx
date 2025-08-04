import useRequest from '@/hooks/useRequest';
import useTokenOrRedirect from '@/hooks/useTokenOrRedirect';
import { UsernamesResponseSchema, type UsernamesResponse } from '@/types';
import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import styles from '@/styles/shared.module.css';

const FriendsPage = () => {
  const token = useTokenOrRedirect();

  const { data, error, loading, sendRequest } = useRequest<null, UsernamesResponse>({
    method: 'GET',
    endpoint: 'friends',
    respSchema: UsernamesResponseSchema,
  });

  useEffect(() => {
    if (token) void sendRequest({ token });
  }, [sendRequest, token]);

  return (
    <>
      <h2>Friends</h2>
      <Link to='/friends/requests'>
        <button type='button'>See pending requests</button>
      </Link>
      <Link to='/friends/add'>
        <button type='button'>Add a new friend</button>
      </Link>
      <hr />
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && (
        <>
          <h3>Current friends</h3>
          {data.usernames.length ? (
            <table>
              <tbody>
                {data.usernames.map(username => (
                  <tr key={username}>
                    <td>{username}</td>
                    <td className={styles.buttonCell}>
                      <Link to={`/friends/${username}`}>
                        <button type='button'>View profile</button>
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p>(No friends)</p>
          )}
        </>
      )}
    </>
  );
};

export default FriendsPage;
