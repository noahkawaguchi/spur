import FriendRequestsPage from '@/pages/Friends/Requests';
import { dummyUsernames } from '@/test-utils/dummy-data';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';

const mockUseRequestResultState = initMockUseRequestResult<string[]>();
const testToken = 'this is a test token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => testToken),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/hooks/useTokenOrRedirect', () => ({ default: mocks.mockUseTokenOrRedirect }));

describe('FriendRequestsPage', () => {
  afterEach(() => vi.clearAllMocks());

  it('should display a no requests message if the usernames array is empty', () => {
    mockUseRequestResultState.data = [];
    render(inMemRouter({ children: <FriendRequestsPage /> }));
    expect(screen.getByText('(No pending friend requests)')).toBeInTheDocument();
  });

  it('should display requester usernames if they exist', () => {
    mockUseRequestResultState.data = dummyUsernames;
    render(inMemRouter({ children: <FriendRequestsPage /> }));
    dummyUsernames.forEach(requester => expect(screen.getByText(requester)).toBeInTheDocument());
  });
});
