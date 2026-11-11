public final class MixedArrayValue {
    static boolean[] values = new boolean[1];
    static boolean bValue;
    static boolean cValue;
    static int arrayCalls;
    static int indexCalls;
    static int bCalls;
    static int cCalls;
    static boolean[] array(boolean nullArray) { arrayCalls++; return nullArray ? null : values; }
    static int index(int value) { indexCalls++; return value; }
    static boolean b() { bCalls++; return bValue; }
    static boolean c() { cCalls++; return cValue; }
    static void one(boolean a, boolean nullArray, int pos) {
        array(nullArray)[index(pos)] = (a && b()) || c();
    }
}
