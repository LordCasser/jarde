public final class BoundaryRunner {
    public static void main(String[] args) throws Exception {
        BoundaryTagged tagged = new BoundaryTagged();
        System.out.println(tagged.wideAndVarargs(2L, 3.0d, "x", "yy"));
        System.out.println(tagged.sameTypeAtDistinctPositions(4, "abc"));
        System.out.println(BoundaryTagged.class.getDeclaredField("field")
                .getDeclaredAnnotations().length);
        System.out.println(BoundaryTagged.class.getDeclaredMethod(
                "wideAndVarargs", long.class, double.class, String[].class)
                .getParameterAnnotations().length);
    }
}
