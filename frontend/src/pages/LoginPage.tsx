import { useEffect, useState } from 'react';
import { setToken } from '../utils/jwt';
import { useNavigate } from 'react-router-dom';
import z from 'zod';
import useRequest from '../hooks/useRequest';

interface LoginRequest {
  email: string;
  password: string;
}

const LoginResponseSchema = z.object({ token: z.string() });
type LoginResponse = z.infer<typeof LoginResponseSchema>;

const LoginPage = () => {
  const navigate = useNavigate();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const { data, error, loading, sendRequest } = useRequest<LoginRequest, LoginResponse>({
    method: 'POST',
    endpoint: 'auth/login',
    respSchema: LoginResponseSchema,
    // Display any login errors such as invalid password instead of looping back to login again
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

      <form onSubmit={handleSubmit}>
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
        <button type='submit' disabled={loading}>
          Submit
        </button>
      </form>

      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {data && <p>Success!</p>}

      <p>or sign up</p>
      <hr />
    </>
  );
};

export default LoginPage;
