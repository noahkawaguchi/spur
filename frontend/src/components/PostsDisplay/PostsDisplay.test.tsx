import PostsDisplay from '@/components/PostsDisplay';
import { dummyPosts } from '@/test-utils/dummy-data';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import type { Post } from '@/types';
import { firstChars } from '@/utils/fmt';
import { render, screen } from '@testing-library/react';

const mockUseRequestResultState = initMockUseRequestResult<Post[]>();
const testToken = 'this is a test token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => testToken),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/hooks/useTokenOrRedirect', () => ({ default: mocks.mockUseTokenOrRedirect }));

describe('PostsDisplay', () => {
  const testHeader = <p>Hello from a test!</p>;
  const testEndpoint = 'my-test-endpoint';
  const testDisplayUsernames = true;

  const renderPostsDisplay = () =>
    render(
      inMemRouter({
        children: (
          <PostsDisplay
            header={testHeader}
            endpoint={testEndpoint}
            displayUsernames={testDisplayUsernames}
          />
        ),
      }),
    );

  afterEach(() => vi.clearAllMocks());

  it('should display a no posts message if the usernames array is empty', () => {
    mockUseRequestResultState.data = [];
    renderPostsDisplay();
    expect(screen.getByText('(No posts)')).toBeInTheDocument();
  });

  it('should display posts if they exist', () => {
    mockUseRequestResultState.data = dummyPosts;
    renderPostsDisplay();
    dummyPosts.forEach(post => {
      expect(screen.getByText(post.authorUsername, { exact: false })).toBeInTheDocument();
      expect(screen.getByText(firstChars(post.body))).toBeInTheDocument();
    });
  });
});
