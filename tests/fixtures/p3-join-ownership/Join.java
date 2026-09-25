public class Join {
    static int mixedExits(int n) {
        int r = 0;
        switch (n) {
            case 1:
                r = 10;
                break;
            case 2:
                return 20;
            default:
                r = 30;
        }
        return r;
    }

    static int deepCatch(int n) {
        try {
            try {
                n = n + 1;
            } catch (RuntimeException e) {
                n = -1;
            }
            n = n * 3;
        } catch (Exception e) {
            n = -9;
        }
        return n;
    }

    static boolean both(int a, int b) {
        return a > 0 && b > 0;
    }

    static int pick(String s) {
        if (s.isEmpty()) {
            return 0;
        }
        return s.length() > 3 ? big(s) : small(s);
    }

    static int big(String s) {
        return s.length() * 10;
    }

    static int small(String s) {
        return s.length();
    }
}
