import { useNavigate } from 'react-router-dom';

const TOKEN_KEY = 'jwt_token';

export const getToken = (): string | null => {
  return localStorage.getItem(TOKEN_KEY);
};

export const useTokenOrRedirect = (): string | undefined => {
  const navigate = useNavigate();
  const token = getToken();
  if (token) return token;
  void navigate('/login');
};

export const setToken = (token: string): void => {
  localStorage.setItem(TOKEN_KEY, token);
};

export const removeToken = (): void => {
  localStorage.removeItem(TOKEN_KEY);
};
