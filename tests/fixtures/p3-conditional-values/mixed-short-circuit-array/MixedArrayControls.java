public final class MixedArrayControls {
    static boolean[] values = new boolean[1];
    static byte[] bytes = new byte[1];
    static boolean[] saved;
    static int lastIndex;

    static boolean[] array() { return values; }
    static byte[] byteArray() { return bytes; }
    static int index(int value) { return value; }
    static boolean b() { return true; }
    static boolean c() { return false; }

    static void byteTarget(boolean a, int pos) {
        byteArray()[index(pos)] = (a && b()) || c() ? (byte) 1 : (byte) 0;
    }

    static void plainWrite(boolean a, int pos) {
        array()[index(pos)] = (a && b()) || c();
    }

    static void nullTarget(boolean a, int pos) {
        ((boolean[]) null)[index(pos)] = (a && b()) || c();
    }

    static void sharedArray(boolean a, int pos) {
        (saved = array())[index(pos)] = (a && b()) || c();
    }

    static void sharedIndex(boolean a, int pos) {
        array()[lastIndex = index(pos)] = (a && b()) || c();
    }

    static boolean duplicatedValue(boolean a, int pos) {
        return array()[index(pos)] = (a && b()) || c();
    }

    static void protectedWrite(boolean a, int pos) {
        try {
            array()[index(pos)] = (a && b()) || c();
        } catch (RuntimeException ignored) {
        }
    }

    static void guardedWrite(boolean guard, boolean a, int pos) {
        if (guard) {
            array()[index(pos)] = (a && b()) || c();
        }
    }
}
