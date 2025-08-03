import LoginPage from '@/pages/Login';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';

const mockUseRequestResultState = initMockUseRequestResult<{ token: string }>();

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockSetToken: vi.fn(),
    mockNavigate: vi.fn(),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/utils/jwt', async () => {
  const actual = await vi.importActual<typeof import('@/utils/jwt')>('@/utils/jwt');
  return { ...actual, setToken: mocks.mockSetToken };
});
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return { ...actual, useNavigate: () => mocks.mockNavigate };
});

describe('LoginPage', () => {
  afterEach(() => vi.clearAllMocks());

  it('should take user input, send a request, and handle the response', async () => {
    const { rerender } = render(inMemRouter({ children: <LoginPage /> }));
    const user = userEvent.setup();

    const email = 'alice@example.com';
    const password = 'secret_password_12345';
    const token = '294852has-df$';

    await user.type(screen.getByLabelText('Email:'), email);
    await user.type(screen.getByLabelText('Password:'), password);
    await user.click(screen.getByText('Submit'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      body: { email, password },
    });

    mockUseRequestResultState.data = { token };
    mockUseRequestResultState.success = true;

    rerender(inMemRouter({ children: <LoginPage /> }));

    expect(mocks.mockSetToken).toHaveBeenCalledExactlyOnceWith(token);
    expect(mocks.mockNavigate).toHaveBeenCalledExactlyOnceWith('/');
  });
});
