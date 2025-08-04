import { useEffect, useState } from 'react';
import { setToken } from '@/utils/jwt';
import { Link, useNavigate } from 'react-router-dom';
import useRequest from '@/hooks/useRequest';
import { TokenResponseSchema, type TokenResponse } from '@/types';
import styles from '@/styles/shared.module.css';

const LoginPage = () => {
  const navigate = useNavigate();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const { data, error, loading, sendRequest } = useRequest<
    { email: string; password: string },
    TokenResponse
  >({
    method: 'POST',
    endpoint: 'auth/login',
    respSchema: TokenResponseSchema,
    redirect401: false,
  });

  useEffect(() => {
    if (data) {
      setToken(data.token);
      void navigate('/');
    }
  }, [data, navigate]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void sendRequest({ body: { email, password } });
  };

  return (
    <>
      <h2>Login</h2>
      <hr />
      <form onSubmit={handleSubmit} className={styles.authForm}>
        <label>
          Email:{' '}
          <input
            type='email'
            value={email}
            onChange={e => {
              setEmail(e.target.value);
            }}
            placeholder='you@email.com'
            disabled={loading}
            required
            autoFocus
          />
        </label>
        <br />
        <label>
          Password:{' '}
          <input
            type='password'
            value={password}
            onChange={e => {
              setPassword(e.target.value);
            }}
            placeholder='**********'
            disabled={loading}
            required
          />
        </label>
        <br />
        <button type='submit' disabled={loading}>
          Submit
        </button>
      </form>
      <p>
        or{' '}
        <Link to='/signup'>
          <button type='button'>sign up</button>
        </Link>{' '}
        for a new account
      </p>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && <p>Success!</p>}
      <hr />
    </>
  );
};

export default LoginPage;
