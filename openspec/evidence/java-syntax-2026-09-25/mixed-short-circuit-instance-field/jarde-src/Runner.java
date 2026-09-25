public final class Runner {
    public static void main(String[] args) {
        for (int nullBit = 0; nullBit <= 1; nullBit++) {
            for (int mask = 0; mask < 8; mask++) {
                boolean a = (mask & 1) != 0;
                boolean b = (mask & 2) != 0;
                boolean c = (mask & 4) != 0;
                boolean isNull = nullBit != 0;
                MixedShortCircuitField.bValue = b;
                MixedShortCircuitField.cValue = c;
                MixedShortCircuitField.bCalls = 0;
                MixedShortCircuitField.cCalls = 0;
                MixedShortCircuitField.receiverCalls = 0;
                MixedShortCircuitField.receiver = new MixedShortCircuitField$Box();
                boolean threw = false;
                try { MixedShortCircuitField.one(a, isNull); }
                catch (NullPointerException npe) { threw = true; }
                boolean stored = MixedShortCircuitField.receiver.result;
                System.out.printf("%d:%d:%s:%s:%d:%d:%d:%s%n", nullBit, mask,
                    threw ? "NPE" : "OK", stored, MixedShortCircuitField.bCalls,
                    MixedShortCircuitField.cCalls, MixedShortCircuitField.receiverCalls,
                    MixedShortCircuitField.receiver == null ? "null" : "box");
            }
        }
    }
}
