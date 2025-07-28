import { getToken } from '@/utils/jwt';
import { useNavigate } from 'react-router-dom';

/** Attempts to retrieve the local JWT, redirecting to the login page if not found. */
const useTokenOrRedirect = (): string | null => {
  const navigate = useNavigate();
  const token = getToken();
  if (!token) void navigate('/login');
  return token;
};

export default useTokenOrRedirect;
