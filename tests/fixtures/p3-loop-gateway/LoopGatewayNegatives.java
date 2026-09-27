package cf08;

public final class LoopGatewayNegatives {
    public static int sharedGateway(int limit) {
        int i = 0;
        while (true) {
            if (i >= limit) break;
            if (i == 2 || i == 3) break;
            i++;
        }
        return i;
    }

    public static int differentTarget(int limit) {
        int i = 0;
        outer: {
            while (true) {
                if (i >= limit) break;
                if (i == 3) break outer;
                i++;
            }
            i += 1;
        }
        return i;
    }

    public static int effectGateway(int limit) {
        int i = 0;
        while (true) {
            if (i >= limit) break;
            if (i == 3) {
                i += 0;
                break;
            }
            i++;
        }
        return i;
    }

    public static int exceptionalGateway(int limit) {
        int i = 0;
        try {
            while (true) {
                if (i >= limit) break;
                if (new int[] { i }[0] == 3) break;
                i++;
            }
        } catch (RuntimeException ignored) {
            return -1;
        }
        return i;
    }
}
