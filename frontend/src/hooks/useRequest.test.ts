import useRequest from '@/hooks/useRequest';
import { inMemRouter } from '@/test-utils/router';
import { act, renderHook, waitFor } from '@testing-library/react';
import { type MockedFunction } from 'vitest';
import z from 'zod';

describe('useRequest', () => {
  let mockFetch: MockedFunction<typeof fetch>;

  beforeEach(() => {
    mockFetch = vi.fn();
    vi.stubGlobal('fetch', mockFetch);
  });

  afterEach(() => vi.unstubAllGlobals());

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
  });
});
