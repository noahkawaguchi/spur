import FriendsPage from '@/pages/Friends';
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

describe('FriendsPage', () => {
  afterEach(() => vi.clearAllMocks());

  it('should display a no friends message if the usernames array is empty', () => {
    mockUseRequestResultState.data = [];
    render(inMemRouter({ children: <FriendsPage /> }));
    expect(screen.getByText('(No friends)')).toBeInTheDocument();
  });

  it('should display friend usernames if they exist', () => {
    const friends = ['jeff1', 'jeff2', 'jefe', 'jpeg'];
    mockUseRequestResultState.data = friends;
    render(inMemRouter({ children: <FriendsPage /> }));
    friends.forEach(friend => expect(screen.getByText(friend)).toBeInTheDocument());
  });
});
