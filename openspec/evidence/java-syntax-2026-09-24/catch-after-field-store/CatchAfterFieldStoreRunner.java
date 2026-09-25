public final class CatchAfterFieldStoreRunner {
    public static void main(String[] args) {
        for (boolean shouldThrow : new boolean[] { false, true }) {
            System.out.println("field=" + CatchAfterFieldStore.fieldStore(shouldThrow)
                    + ",local=" + CatchAfterFieldStore.localStore(shouldThrow));
        }
    }
}
