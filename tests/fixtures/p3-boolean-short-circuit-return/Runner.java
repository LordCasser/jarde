public final class Runner {
    public static void main(String[] args) {
        for (int a : new int[] {-1, 1}) {
            for (int b : new int[] {-1, 1}) {
                BoolValue.calls = 0;
                boolean and = BoolValue.effectfulAnd(a, b);
                int andCalls = BoolValue.calls;
                BoolValue.calls = 0;
                boolean or = BoolValue.effectfulOr(a, b);
                int orCalls = BoolValue.calls;
                System.out.println(a + "," + b + ":" + BoolValue.and(a, b) + ","
                        + BoolValue.or(a, b) + "," + and + "," + andCalls + "," + or
                        + "," + orCalls);
            }
        }
    }
}
