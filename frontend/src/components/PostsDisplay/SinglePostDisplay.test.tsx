import SinglePostDisplay from '@/components/PostsDisplay/SinglePostDisplay';
import { dummyPosts } from '@/test-utils/dummy-data';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import type { Post } from '@/types';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const mockUseRequestResultState = initMockUseRequestResult<Post>();
const testToken = 'this is a test token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => testToken),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/hooks/useTokenOrRedirect', () => ({ default: mocks.mockUseTokenOrRedirect }));

describe('SinglePostDisplay', () => {
  const testReadingPost = dummyPosts[0];
  const testParentPost = dummyPosts[1];
  const mockBackFn = vi.fn();

  const spdInRouter = () =>
    inMemRouter({
      children: <SinglePostDisplay readingPost={testReadingPost} backFn={mockBackFn} />,
    });

  afterEach(() => vi.clearAllMocks());

  it('should call the back function when the back button is clicked', async () => {
    render(spdInRouter());
    const user = userEvent.setup();
    await user.click(screen.getByText('Back'));
    expect(mockBackFn).toHaveBeenCalledOnce();
  });

  it('should show the reply box when the reply button is clicked', async () => {
    render(spdInRouter());
    const user = userEvent.setup();
    await user.click(screen.getByText('Reply'));
    expect(screen.getByLabelText('New Reply:')).toBeInTheDocument();
  });

  it('should make a request and show the parent when the parent button is clicked', async () => {
    const { rerender } = render(spdInRouter());
    expect(screen.queryByText(testParentPost.body)).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByText('Parent'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      token: testToken,
      pathParameter: testReadingPost.parentId?.toString(),
    });

    mockUseRequestResultState.data = testParentPost;
    rerender(spdInRouter());
    expect(screen.queryByText(testParentPost.body)).toBeInTheDocument();
  });

  it(
    'should render the component to display the children ' + 'when the children button is clicked',
    async () => {
      const childrenText = `Children of ${testReadingPost.authorUsername}'s Post`;

      render(spdInRouter());
      expect(screen.queryByText(childrenText)).not.toBeInTheDocument();

      const user = userEvent.setup();
      await user.click(screen.getByText('Children'));

      expect(screen.queryByText(childrenText)).toBeInTheDocument();
    },
  );
});
