import useRequest from '@/hooks/useRequest';
import { inMemRouter } from '@/test-utils/router';
import { act, renderHook, waitFor } from '@testing-library/react';
import { type MockedFunction } from 'vitest';
import z from 'zod';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

describe('useRequest', () => {
  let mockFetch: MockedFunction<typeof fetch>;

  const renderSimpleVersion = () =>
    renderHook(() => useRequest({ method: 'GET', endpoint: 'dummy', respSchema: null }), {
      wrapper: inMemRouter,
    });

  beforeEach(() => {
    mockFetch = vi.fn();
    vi.stubGlobal('fetch', mockFetch);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    mockNavigate.mockClear();
  });

  it('should make a simple request with no options provided', async () => {
    const respBody = { hello: 'world' };

    mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(respBody), { status: 200 }));

    const { result } = renderHook(
      () =>
        useRequest({
          method: 'GET',
          endpoint: 'dummy',
          respSchema: z.object({ hello: z.string() }),
        }),
      { wrapper: inMemRouter },
    );

    await act(() => result.current.sendRequest({}));
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
    expect(result.current.data).toStrictEqual(respBody);
    expect(result.current.success).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('should apply request options when provided', async () => {
    const reqBody = { using: 'options' };
    const respBody = { nowUse: 'some options' };
    const token = '53cr3t';

    mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(respBody), { status: 200 }));

    const { result } = renderHook(
      () =>
        useRequest({
          method: 'POST',
          endpoint: 'dummy/with-options',
          respSchema: z.object({ nowUse: z.string() }),
        }),
      { wrapper: inMemRouter },
    );

    await act(() =>
      result.current.sendRequest({ pathParameter: 'more-options', token, body: reqBody }),
    );
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy/with-options/more-options', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify(reqBody),
    });
    expect(result.current.data).toStrictEqual(respBody);
    expect(result.current.success).toBe(true);
    expect(result.current.error).toBeNull();
  });

  describe('useRequest error handling', () => {
    it('should handle standard JSON error responses from the server', async () => {
      const errMsg = 'something went wrong!';

      mockFetch.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: errMsg }), { status: 418 }),
      );

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
      expect(result.current.data).toBeNull();
      expect(result.current.success).toBe(false);
      expect(result.current.error).toStrictEqual(errMsg);
    });

    it('should handle success responses with bad bodies', async () => {
      mockFetch.mockResolvedValueOnce(
        new Response(JSON.stringify({ you: 'thought' }), { status: 200 }),
      );

      const { result } = renderHook(
        () =>
          useRequest({
            method: 'GET',
            endpoint: 'dummy',
            respSchema: z.object({ expecting: z.string() }),
          }),
        { wrapper: inMemRouter },
      );

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
      expect(result.current.data).toBeNull();
      expect(result.current.success).toBe(true);
      expect(result.current.error).toStrictEqual('success response but unexpected body type');
    });

    it('should handle text error responses', async () => {
      const errText = 'ahh!!';
      mockFetch.mockResolvedValueOnce(new Response(errText, { status: 404 }));

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
      expect(result.current.data).toBeNull();
      expect(result.current.success).toBe(false);
      expect(result.current.error).toStrictEqual(errText);
    });

    it('should handle responses with only status text', async () => {
      const statusText = "I'm a teapot";
      mockFetch.mockResolvedValueOnce(new Response(undefined, { status: 418, statusText }));

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
      expect(result.current.data).toBeNull();
      expect(result.current.success).toBe(false);
      expect(result.current.error).toStrictEqual(statusText);
    });

    it('should fall back to the status code', async () => {
      mockFetch.mockResolvedValueOnce(new Response(undefined, { status: 404 }));

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockFetch).toHaveBeenCalledExactlyOnceWith('undefined/dummy', { method: 'GET' });
      expect(result.current.data).toBeNull();
      expect(result.current.success).toBe(false);
      expect(result.current.error).toStrictEqual(`HTTP 404`);
    });
  });

  describe('useRequest redirection logic', () => {
    it('should redirect for 401 when redirect401 is true', async () => {
      mockFetch.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'bad token' }), { status: 401 }),
      );

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockNavigate).toHaveBeenCalledExactlyOnceWith('/login');
    });

    it('should not redirect for 401 when redirect401 is false', async () => {
      mockFetch.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'bad token' }), { status: 401 }),
      );

      const { result } = renderHook(
        () =>
          useRequest({
            method: 'POST',
            endpoint: 'dummy/no-redirect-please',
            respSchema: null,
            redirect401: false,
          }),
        { wrapper: inMemRouter },
      );

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockNavigate).not.toHaveBeenCalled();
    });

    it('should not redirect for responses other than 401', async () => {
      mockFetch.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'that is not allowed' }), { status: 403 }),
      );

      const { result } = renderSimpleVersion();

      await act(() => result.current.sendRequest({}));
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockNavigate).not.toHaveBeenCalled();
    });
  });
});
