final class ArrayForeachRefusalRunner {
    public static void main(String[] args) {
        System.out.println("index=" + ArrayForeachRefusal.indexInBody(new int[] {3, 4}));
        System.out.println("escape=" + ArrayForeachRefusal.indexAfterLoop(new int[] {3, 4}));
        System.out.println("effect=" + ArrayForeachRefusal.effectInBinding(new int[] {3, 4}));
        System.out.println("element=" + ArrayForeachRefusal.escapedElement(new int[] {3, 4}));
        try {
            ArrayForeachRefusal.differentArrays(new int[] {1, 2}, new int[] {9});
            System.out.println("mismatch=NORMAL");
        } catch (Throwable error) {
            System.out.println("mismatch=" + error.getClass().getSimpleName());
        }
    }
}
