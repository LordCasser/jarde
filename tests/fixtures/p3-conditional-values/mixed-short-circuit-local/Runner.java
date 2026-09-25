public final class Runner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 8; mask++) {
            boolean a = (mask & 4) != 0;
            boolean b = (mask & 2) != 0;
            boolean c = (mask & 1) != 0;
            MixedBooleanLocal.bValue = b;
            MixedBooleanLocal.cValue = c;
            MixedBooleanLocal.result = false;
            MixedBooleanLocal.bCalls = 0;
            MixedBooleanLocal.cCalls = 0;

            boolean value = MixedBooleanLocal.one(a);
            System.out.printf("%d:%s:%s:%d:%d%n", mask, value,
                    MixedBooleanLocal.result, MixedBooleanLocal.bCalls,
                    MixedBooleanLocal.cCalls);
        }
    }
}
