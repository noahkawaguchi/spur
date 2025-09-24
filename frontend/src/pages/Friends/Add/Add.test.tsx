import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { SuccessResponse } from '@/types';
import AddFriendPage from '@/pages/Friends/Add';

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

describe('AddFriendPage', () => {
  const testUsername = 'danny-mann0';
  const testMessage = 'yay!!!';

  afterEach(() => vi.clearAllMocks());

  it('should take user input, send a request, and handle the response', async () => {
    const { rerender } = render(inMemRouter({ children: <AddFriendPage /> }));
    const user = userEvent.setup();

    await user.type(screen.getByLabelText('Username:'), testUsername);
    await user.click(screen.getByText('Submit'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      token: testToken,
      body: { recipientUsername: testUsername },
    });

    mockUseRequestResultState.data = { message: testMessage };
    rerender(inMemRouter({ children: <AddFriendPage /> }));
    expect(screen.getByText(testMessage)).toBeInTheDocument();
  });
});
