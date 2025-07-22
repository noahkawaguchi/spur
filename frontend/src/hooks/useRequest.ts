import { useState, useCallback, useRef } from 'react';
import { backendUrl } from '../config';
import { ZodType, z } from 'zod';
import { useNavigate } from 'react-router-dom';
import { removeToken } from '../utils/jwt';

interface ReqOpts<T> {
  pathParameter?: string;
  token?: string;
  body?: T;
}

/**
 * Custom hook for making HTTP requests to the backend.
 *
 * @template TResponse - The expected response type.
 * @template TRequest  - The type of the request body (optional).
 *
 * @param method     - The HTTP method to use.
 * @param endpoint   - The API endpoint. Do not include the base URL, leading slash, or parameters.
 * @param respSchema - The schema for the expected response type.
 *
 * @returns { data, error, loading, sendRequest }
 *          data        - The expected TResponse or null.
 *          error       - Any error encountered or null.
 *          loading     - Whether the request is currently in progress.
 *          sendRequest - The function to trigger the request.
 */
const useRequest = <TResponse, TRequest = undefined>(
  method: 'GET' | 'POST',
  endpoint: string,
  respSchema: ZodType<TResponse> | null,
): {
  data: TResponse | null;
  error: string | null;
  loading: boolean;
  sendRequest: (opts: ReqOpts<TRequest>) => Promise<void>;
} => {
  const [data, setData] = useState<TResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Ref to persist across rerenders and avoid double requests, especially when using React's
  // StrictMode in development
  const isSubmitting = useRef(false);

  const navigate = useNavigate();

  /**
   * Triggers a request using the specified options (all optional).
   * If the backend responds with 401 Unauthorized, removes the stored JWT and redirects to the
   * login page.
   *
   * @param pathParameter - The desired path parameter. Do not include a slash.
   * @param token         - The JSON Web Token to be sent in the Authorization header.
   * @param body          - The body to be sent in the request.
   */
  const sendRequest = useCallback(
    async (opts: ReqOpts<TRequest>) => {
      if (isSubmitting.current) return;
      isSubmitting.current = true;

      setData(null);
      setError(null);
      setLoading(true);

      const url = opts.pathParameter
        ? `${backendUrl}/${endpoint}/${opts.pathParameter}`
        : `${backendUrl}/${endpoint}`;

      const headers: Record<string, string> = {};
      if (opts.body) headers['Content-Type'] = 'application/json';
      if (opts.token) headers.Authorization = `Bearer ${opts.token}`;

      await fetch(url, {
        method,
        headers: Object.keys(headers).length ? headers : undefined,
        body: JSON.stringify(opts.body), // undefined if opts.body is undefined
      })
        .then(async response => {
          if (response.ok) {
            // Only attempt to parse the response body if one is expected
            if (respSchema) {
              const parsedBody = respSchema.safeParse(await response.json());
              if (parsedBody.success) setData(parsedBody.data);
              else throw new Error('success response but unexpected body type');
            }
          } else if (response.status === 401) {
            // Token expired
            removeToken();
            void navigate('/login');
          } else {
            const parsedErrBody = z.object({ error: z.string() }).safeParse(await response.json());
            throw new Error(parsedErrBody.success ? parsedErrBody.data.error : response.statusText);
          }
        })
        .catch((err: unknown) => {
          setError(err instanceof Error ? err.message : 'unknown error');
        })
        .finally(() => {
          setLoading(false);
          isSubmitting.current = false;
        });
    },
    [endpoint, method, respSchema, navigate],
  );

  return { data, error, loading, sendRequest };
};

export default useRequest;
