public final class Runner {
    public static void main(String[] args) {
        NestedArrayInitializer.calls = 0;
        NestedArrayInitializer.trace = 0;
        System.out.println(java.util.Arrays.deepToString(NestedArrayInitializer.dynamic())
            + ":" + NestedArrayInitializer.calls + ":" + NestedArrayInitializer.trace);
        System.out.println(java.util.Arrays.deepToString(NestedArrayInitializer.literal()));
    }
}
