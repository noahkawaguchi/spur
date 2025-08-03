import { type Mock } from 'vitest';

interface MockUseRequestResult<T> {
  data: null | T;
  success: boolean;
  error: null | string;
  loading: boolean;
  sendRequest: Mock;
}

export const initMockUseRequestResult = <T>(): MockUseRequestResult<T> => {
  return { data: null, success: false, error: null, loading: false, sendRequest: vi.fn() };
};
