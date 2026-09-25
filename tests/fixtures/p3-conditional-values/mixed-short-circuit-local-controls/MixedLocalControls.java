public final class MixedLocalControls {
    static boolean bValue;
    static boolean cValue;
    static boolean result;

    static boolean b() { return bValue; }
    static boolean c() { return cValue; }

    static int numeric(boolean a) {
        int value = (a && b()) || c() ? 1 : 0;
        return value + 1;
    }

    static boolean rewritten(boolean a) {
        boolean value = (a && b()) || c();
        value = !value;
        result = value;
        return value;
    }

    static boolean duplicated(boolean a) {
        boolean value = result = (a && b()) || c();
        return value;
    }

    static boolean crossing(boolean a) {
        boolean value;
        try {
            value = (a && b()) || c();
        } catch (RuntimeException error) {
            value = false;
        }
        result = value;
        return value;
    }
}
