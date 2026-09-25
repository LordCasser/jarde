public class LoopTryHandlerEntry {
    static int maybeFail(int n, int failAt) {
        if (n == failAt) {
            throw new IllegalStateException();
        }
        return n;
    }

    static int loopTry(int n, int failAt) {
        int result = 0;
        while (n > 0) {
            try {
                result = result + maybeFail(n, failAt);
            } catch (RuntimeException e) {
                result = -1;
            }
            n = n - 1;
        }
        return result;
    }
}
