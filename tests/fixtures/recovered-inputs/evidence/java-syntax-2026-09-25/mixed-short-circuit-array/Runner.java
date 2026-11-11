public final class Runner {
    public static void main(String[] args) {
        for (int bits = 0; bits < 8; bits++) {
            for (int kind = 0; kind < 3; kind++) {
                MixedArrayValue.bValue = (bits & 2) != 0;
                MixedArrayValue.cValue = (bits & 4) != 0;
                MixedArrayValue.values[0] = false;
                MixedArrayValue.arrayCalls = MixedArrayValue.indexCalls = MixedArrayValue.bCalls = MixedArrayValue.cCalls = 0;
                boolean a = (bits & 1) != 0;
                boolean nil = kind == 1;
                int pos = kind == 2 ? 2 : 0;
                String outcome;
                try {
                    MixedArrayValue.one(a, nil, pos);
                    outcome = "OK";
                } catch (Throwable ex) {
                    outcome = ex.getClass().getSimpleName();
                }
                System.out.println(bits + ":" + kind + ":" + outcome + ":" + MixedArrayValue.values[0] + ":" + MixedArrayValue.arrayCalls + ":" + MixedArrayValue.indexCalls + ":" + MixedArrayValue.bCalls + ":" + MixedArrayValue.cCalls);
            }
        }
    }
}
