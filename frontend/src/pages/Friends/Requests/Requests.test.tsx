import FriendRequestsPage from '@/pages/Friends/Requests';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';

const mockUseRequestResultState = initMockUseRequestResult<string[]>();
const mockToken = 'this is a mock token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => mockToken),
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
    const requesters = ['jeff1', 'jeff2', 'jefe', 'jpeg'];
    mockUseRequestResultState.data = requesters;
    render(inMemRouter({ children: <FriendRequestsPage /> }));
    requesters.forEach(requester => expect(screen.getByText(requester)).toBeInTheDocument());
  });
});
