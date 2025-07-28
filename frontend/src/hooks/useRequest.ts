import { useState, useCallback, useRef } from 'react';
import { ZodType, z } from 'zod';
import { useNavigate } from 'react-router-dom';
import { backendUrl } from '@/config';
import { removeToken } from '@/utils/jwt';

interface UseRequestOpts<TResponse> {
  method: 'GET' | 'POST';
  endpoint: string;
  respSchema: ZodType<TResponse> | null;
  redirect401?: boolean;
}

interface UseRequestResult<TRequest, TResponse> {
  data: TResponse | null;
  success: boolean;
  error: string | null;
  loading: boolean;
  sendRequest: (opts: SendRequestOpts<TRequest>) => Promise<void>;
}

interface SendRequestOpts<TRequest> {
  pathParameter?: string;
  token?: string;
  body?: TRequest;
}

/**
 * Custom hook for making HTTP requests to the backend.
 *
 * @template TRequest  - The type of the request body (pass null for no body).
 * @template TResponse - The type of the expected response body (pass null for no body).
 *
 * @param opts             - Object containing hook options.
 * @param opts.method      - The HTTP method to use.
 * @param opts.endpoint    - The API endpoint. Do not include the base URL, leading slash, or path
 *                           parameters.
 * @param opts.respSchema  - The schema for the expected response type.
 * @param opts.redirect401 - Whether to remove the local token and redirect to the login page if
 *                           the backend responds with 401 Unauthorized. Defaults to true.
 *
 * @returns { data, success, error, loading, sendRequest }
 *          data        - The expected TResponse or null.
 *          success     - Whether the request received a success response. Can be used in place of
 *                        data when no response body is expected.
 *          error       - Any error encountered or null.
 *          loading     - Whether the request is currently in progress.
 *          sendRequest - The function to trigger the request.
 */
const useRequest = <TRequest, TResponse>({
  method,
  endpoint,
  respSchema,
  redirect401 = true,
}: UseRequestOpts<TResponse>): UseRequestResult<TRequest, TResponse> => {
  const [data, setData] = useState<TResponse | null>(null);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Ref to persist across rerenders and avoid double requests, especially when using React's
  // StrictMode in development
  const isSubmitting = useRef(false);

  const navigate = useNavigate();

  /**
   * Triggers a request using the specified options, if any.
   *
   * @param opts               - Object containing request options (all optional).
   * @param opts.pathParameter - The desired path parameter. Do not include a slash.
   * @param opts.token         - The JSON Web Token to be sent in the Authorization header.
   * @param opts.body          - The body to be sent in the request.
   */
  const sendRequest = useCallback(
    async ({ pathParameter, token, body }: SendRequestOpts<TRequest>) => {
      if (isSubmitting.current) return;
      isSubmitting.current = true;

      setData(null);
      setSuccess(false);
      setError(null);
      setLoading(true);

      const url = pathParameter
        ? `${backendUrl}/${endpoint}/${pathParameter}`
        : `${backendUrl}/${endpoint}`;

      const headers: Record<string, string> = {};
      if (body) headers['Content-Type'] = 'application/json';
      if (token) headers.Authorization = `Bearer ${token}`;

      await fetch(url, {
        method,
        headers: Object.keys(headers).length ? headers : undefined,
        body: JSON.stringify(body), // undefined if opts.body is undefined
      })
        .then(async response => {
          if (response.ok) {
            setSuccess(true);
            // Only attempt to parse the response body if one is expected
            if (respSchema) {
              const parsedBody = respSchema.safeParse(await response.json());
              if (parsedBody.success) setData(parsedBody.data);
              else throw new Error('success response but unexpected body type');
            }
          } else if (redirect401 && response.status === 401) {
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
    [endpoint, method, respSchema, navigate, redirect401],
  );

  return { data, success, error, loading, sendRequest };
};

export default useRequest;
