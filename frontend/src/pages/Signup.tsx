import useRequest from '@/hooks/useRequest';
import { TokenResponseSchema, type TokenResponse } from '@/types';
import { setToken } from '@/utils/jwt';
import { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import styles from '@/styles/shared.module.css';

const SignupPage = () => {
  const navigate = useNavigate();

  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  const { data, error, loading, sendRequest } = useRequest<
    { name: string; email: string; username: string; password: string },
    TokenResponse
  >({
    method: 'POST',
    endpoint: 'auth/signup',
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
    void sendRequest({ body: { name, email, username, password } });
  };

  return (
    <>
      <h2>Signup</h2>
      <hr />
      <form onSubmit={handleSubmit} className={styles.authForm}>
        <label>
          Name:{' '}
          <input
            value={name}
            onChange={e => {
              setName(e.target.value);
            }}
            placeholder='Your Name'
            disabled={loading}
            required
            autoFocus
          />
        </label>
        <br />
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
          />
        </label>
        <br />
        <label>
          Username:{' '}
          <input
            value={username}
            onChange={e => {
              setUsername(e.target.value);
            }}
            placeholder='your_username123'
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
        <Link to='/login'>
          <button type='button'>log in</button>
        </Link>{' '}
        to an existing account
      </p>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && <p>Success!</p>}
      <hr />
    </>
  );
};

export default SignupPage;
