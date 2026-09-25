public final class Runner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 8; mask++) {
            MixedLocalReturn.bValue = (mask & 2) != 0;
            MixedLocalReturn.cValue = (mask & 4) != 0;
            MixedLocalReturn.bCalls = 0;
            MixedLocalReturn.cCalls = 0;
            boolean result = MixedLocalReturn.value((mask & 1) != 0);
            System.out.println(mask + ":" + result + ":" + MixedLocalReturn.bCalls + ":" + MixedLocalReturn.cCalls);
        }
    }
}
