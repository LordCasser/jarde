public class SlotReuseBoundaries {
    public static int sameType(int seed) {
        int sum = 0;
        {
            int first = seed + 1;
            sum += first;
        }
        {
            int second = seed + 2;
            sum += second;
        }
        return sum;
    }

    public static int exclusiveBranch(boolean chooseArray) {
        int result;
        if (chooseArray) {
            int[] local = new int[] { 7 };
            result = local[0];
        } else {
            int local = 9;
            result = local;
        }
        return result;
    }

    public static int loopBodyReuse(int count) {
        int sum = 0;
        while (count > 0) {
            {
                int[] values = new int[] { count };
                sum += values[0];
            }
            {
                int item = count + 1;
                sum += item;
            }
            count--;
        }
        return sum;
    }

    public static int handlerReuse(boolean throwIt) {
        int result = 0;
        try {
            {
                int[] values = new int[] { 3 };
                result += values[0];
            }
            if (throwIt) {
                throw new IllegalStateException();
            }
        } catch (IllegalStateException caught) {
            int value = 5;
            result += value;
        }
        return result;
    }

    public static long category2Adjacent() {
        {
            int[] values = new int[] { 2 };
            if (values[0] == 0) {
                return 0L;
            }
        }
        long wide = 0x100000002L;
        int tail = (int) wide;
        return wide + tail;
    }
}
