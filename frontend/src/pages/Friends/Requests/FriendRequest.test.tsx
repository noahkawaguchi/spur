import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';
import FriendRequest from '@/pages/Friends/Requests/FriendRequest';
import userEvent from '@testing-library/user-event';
import type { SuccessResponse } from '@/types';

const mockUseRequestResultState = initMockUseRequestResult<SuccessResponse>();
const testToken = 'this is a test token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => testToken),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/hooks/useTokenOrRedirect', () => ({ default: mocks.mockUseTokenOrRedirect }));

describe('FriendRequest', () => {
  const testUsername = 'danny-mann0';
  const testMessage = 'yay!!!';

  const reqInTableInRouter = () =>
    inMemRouter({
      children: (
        <table>
          <tbody>
            <FriendRequest username={testUsername} />
          </tbody>
        </table>
      ),
    });

  afterEach(() => vi.clearAllMocks());

  it('should take user input, send a request, and handle the response', async () => {
    const { rerender } = render(reqInTableInRouter());

    const user = userEvent.setup();
    expect(screen.getByText(testUsername)).toBeInTheDocument();
    await user.click(screen.getByText('Accept'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      token: testToken,
      body: { recipientUsername: testUsername },
    });

    mockUseRequestResultState.data = { message: testMessage };
    rerender(reqInTableInRouter());
    expect(screen.getByText(testMessage)).toBeInTheDocument();
  });
});
