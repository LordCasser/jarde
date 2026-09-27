package cf08twolvl;

public final class TwoLevelIfNegatives {
    static int calls;
    static int cost(int value) { calls++; return value; }

    public static int extraEntry(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i;
                if (calls == 100) i = 1;
                else i = 0;
                while (true) {
                    if (i >= length) { result = cost(7); break; }
                    result = xs[i];
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }

    public static int differentTarget(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i = 0;
                while (true) {
                    if (i >= length) { result = cost(7); return result + calls; }
                    result = xs[i];
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }

    public static int bypassJoin(int[] xs) {
        int result;
        outer: {
            if (xs == null) result = -1;
            else {
                int length = xs.length;
                if (length == 0) result = -1;
                else {
                    int i = 0;
                    while (true) {
                        if (i >= length) { result = cost(7); break outer; }
                        result = xs[i];
                        if (result == 3) break;
                        i++;
                    }
                }
                result += calls;
            }
        }
        return result;
    }

    public static int fourthJoinInput(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i = 0;
                if (calls == 100) result = 2;
                else while (true) {
                    if (i >= length) { result = cost(7); break; }
                    result = xs[i];
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }

    public static int doubleCall(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i = 0;
                while (true) {
                    if (i >= length) { result = cost(cost(7)); break; }
                    result = xs[i];
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }

    public static int withHandler(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i = 0;
                while (true) {
                    if (i >= length) {
                        try { result = cost(7); }
                        catch (RuntimeException ex) { result = 8; }
                        break;
                    }
                    result = xs[i];
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }

    public static int extraConsumer(int[] xs) {
        int result;
        if (xs == null) result = -1;
        else {
            int length = xs.length;
            if (length == 0) result = -1;
            else {
                int i = 0;
                while (true) {
                    if (i >= length) { result = cost(7); break; }
                    result = xs[i];
                    if (result == 4) calls += 0;
                    if (result == 3) break;
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }
}
