final class ArrayWrappedBinding {
    static int calls;
    static int[] watched;
    static RuntimeException failure;

    static int tick() {
        calls++;
        if (failure != null) {
            throw failure;
        }
        return calls;
    }

    static int mutateAndTick() {
        calls++;
        watched[0] = 90 + calls;
        if (failure != null) {
            throw failure;
        }
        return calls;
    }

    static int plusArrayFirst(int[] a) {
        int sum = 0;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int value = captured[i] + tick();
            sum += value;
        }
        return sum;
    }

    static int plusTickFirst(int[] a) {
        int sum = 0;
        int[] captured = a;
        watched = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int value = mutateAndTick() + captured[i];
            sum += value;
        }
        return sum;
    }

    static int callTickThenArray(int[] a) {
        int sum = 0;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            consume(tick(), captured[i]);
            sum += last;
        }
        return sum;
    }

    static int callMutateThenArray(int[] a) {
        int sum = 0;
        watched = a;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            consume(mutateAndTick(), captured[i]);
            sum += last;
        }
        return sum;
    }

    static int readAfterEffect(int[] a) {
        int sum = 0;
        watched = a;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            mutateAndTick();
            int value = captured[i];
            sum += value;
        }
        return sum;
    }

    static int twoReadsWithMutation(int[] a) {
        int sum = 0;
        watched = a;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int first = captured[i];
            mutateAndTick();
            int second = captured[i];
            sum += first + second;
        }
        return sum;
    }

    static int wrappedOnlyRead(int[] a) {
        int sum = 0;
        int[] captured = a;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            sum += captured[i] + tick();
        }
        return sum;
    }

    static int last;
    static void consume(int x, int y) { last = x * 1000 + y; }
}
