import SignupPage from '@/pages/Signup';
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

describe('SignupPage', () => {
  afterEach(() => vi.clearAllMocks());

  it('should take user input, send a request, and handle the response', async () => {
    const { rerender } = render(inMemRouter({ children: <SignupPage /> }));
    const user = userEvent.setup();

    const name = 'Alice User';
    const email = 'alice@example.com';
    const username = 'alice123';
    const password = 'secret_password_12345';
    const token = '294852has-df$';

    await user.type(screen.getByLabelText('Name:'), name);
    await user.type(screen.getByLabelText('Email:'), email);
    await user.type(screen.getByLabelText('Username:'), username);
    await user.type(screen.getByLabelText('Password:'), password);
    await user.click(screen.getByText('Submit'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      body: { name, email, username, password },
    });

    mockUseRequestResultState.data = { token };
    mockUseRequestResultState.success = true;

    rerender(inMemRouter({ children: <SignupPage /> }));

    expect(mocks.mockSetToken).toHaveBeenCalledExactlyOnceWith(token);
    expect(mocks.mockNavigate).toHaveBeenCalledExactlyOnceWith('/');
  });

  it('should reject usernames with illegal characters', async () => {
    render(inMemRouter({ children: <SignupPage /> }));
    const user = userEvent.setup();

    await user.type(screen.getByLabelText('Name:'), 'Alice User');
    await user.type(screen.getByLabelText('Email:'), 'alice@example.com');
    await user.type(screen.getByLabelText('Password:'), 'secret_password_12345');

    const usernameInput = screen.getByLabelText('Username:');

    const badUsernames = [
      'w^33b',
      '8()',
      'やっほー',
      'space space',
      'spacious　space',
      'abc\t123',
      'hello🥸789',
      '~-_-~',
    ];

    for (const username of badUsernames) {
      expect(usernameInput).toHaveValue('');
      await user.type(usernameInput, username);
      expect(usernameInput).toHaveValue(username);
      expect(usernameInput).toBeInvalid();
      expect(usernameInput).toBeInstanceOf(HTMLInputElement);
      if (usernameInput instanceof HTMLInputElement) {
        expect(usernameInput.validationMessage).toBe(
          'username may only contain ASCII letters, numbers, underscores, and hyphens',
        );
      }
      await user.clear(usernameInput);
    }
  });
});
