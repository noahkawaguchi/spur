import ReplyWriter from '@/components/PostsDisplay/ReplyWriter';
import { inMemRouter } from '@/test-utils/router';
import { initMockUseRequestResult } from '@/test-utils/types';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const mockUseRequestResultState = initMockUseRequestResult<null>();
const testToken = 'this is a test token';

const mocks = vi.hoisted(() => {
  return {
    mockUseRequest: vi.fn(() => mockUseRequestResultState),
    mockUseTokenOrRedirect: vi.fn(() => testToken),
  };
});

vi.mock('@/hooks/useRequest', () => ({ default: mocks.mockUseRequest }));
vi.mock('@/hooks/useTokenOrRedirect', () => ({ default: mocks.mockUseTokenOrRedirect }));

describe('ReplyWriter', () => {
  const testPostBody = 'hello wide world this is my world wide post';
  const testParentId = 92;
  const mockCancelFn = vi.fn();

  const writerInRouter = () =>
    inMemRouter({ children: <ReplyWriter parentId={testParentId} cancelFn={mockCancelFn} /> });

  afterEach(() => vi.clearAllMocks());

  it('should call the cancel function when the cancel button is clicked', async () => {
    render(writerInRouter());
    const user = userEvent.setup();
    await user.click(screen.getByText('Cancel'));
    expect(mockCancelFn).toHaveBeenCalledOnce();
  });

  it('should take user input, send a request, and handle the response', async () => {
    const { rerender } = render(writerInRouter());
    expect(screen.queryByText('Successfully created!')).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.type(screen.getByLabelText('New Reply:'), testPostBody);
    await user.click(screen.getByText('Post'));

    expect(mockUseRequestResultState.sendRequest).toHaveBeenCalledExactlyOnceWith({
      token: testToken,
      body: { parentId: testParentId, body: testPostBody },
    });

    mockUseRequestResultState.success = true;
    rerender(writerInRouter());
    expect(screen.queryByText('Successfully created!')).toBeInTheDocument();
  });
});
