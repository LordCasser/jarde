public final class Runner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 8; mask++) {
            boolean a = (mask & 1) != 0;
            boolean b = (mask & 2) != 0;
            boolean c = (mask & 4) != 0;
            for (int kind = 0; kind < 2; kind++) {
                MixedBooleanField.result = false;
                MixedBooleanField.bValue = b;
                MixedBooleanField.cValue = c;
                MixedBooleanField.bCalls = 0;
                MixedBooleanField.cCalls = 0;
                if (kind == 0) {
                    MixedBooleanField.andOr(a);
                } else {
                    MixedBooleanField.orAnd(a);
                }
                System.out.println(kind + ":" + mask + ":" + MixedBooleanField.result
                        + ":" + MixedBooleanField.bCalls + ":" + MixedBooleanField.cCalls);
            }
        }
    }
}
