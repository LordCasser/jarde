public final class MixedShortCircuitField {
    static boolean bValue;
    static boolean cValue;
    static int bCalls;
    static int cCalls;
    static int receiverCalls;
    static Box receiver;

    static final class Box {
        boolean result;
    }

    static Box target(boolean returnNull) {
        receiverCalls++;
        return returnNull ? null : receiver;
    }

    static void one(boolean a, boolean nullReceiver) {
        target(nullReceiver).result = (a && b()) || c();
    }

    static boolean b() { bCalls++; return bValue; }
    static boolean c() { cCalls++; return cValue; }
}
