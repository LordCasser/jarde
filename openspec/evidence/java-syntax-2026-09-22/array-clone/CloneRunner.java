public final class CloneRunner {
    public static void main(String[] args) {
        int[] ints = new int[] {2, 3};
        int[] intCopy = CloneProbe.ints(ints);
        System.out.println("ints=" + intCopy[0] + ":" + intCopy[1] + ":distinct=" + (intCopy != ints));

        String[] refs = new String[] {"a", "b"};
        String[] refCopy = CloneProbe.refs(refs);
        System.out.println("refs=" + refCopy[0] + ":" + refCopy[1] + ":distinct=" + (refCopy != refs));

        int[][] matrix = new int[][] {{4}};
        int[][] outerCopy = CloneProbe.outer(matrix);
        System.out.println("outer=distinct:" + (outerCopy != matrix) + ":innerSame:" + (outerCopy[0] == matrix[0]));

        Object[] widened = CloneProbe.widen(refs);
        System.out.println("widen=" + widened[0] + ":type=" + widened.getClass().getName());
        System.out.println("overload=" + CloneProbe.overload(ints));

        try {
            CloneProbe.ints(null);
            System.out.println("null=returned");
        } catch (NullPointerException expected) {
            System.out.println("null=NPE");
        }
    }
}
