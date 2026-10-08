public class LGLambda {
    static int[] pqLambda(int[] arg0) {
        java.util.PriorityQueue local1;
        int[] local2;
        int local3;
        local1 = new java.util.PriorityQueue((java.util.Comparator) ((java.lang.Object p0, java.lang.Object p1) -> ((java.lang.Integer) p1).intValue() - ((java.lang.Integer) p0).intValue()));
        local2 = arg0;
        for (int local5 : local2) {
            local1.offer((java.lang.Object) java.lang.Integer.valueOf(local5));
        }
        local2 = new int[arg0.length];
        local3 = 0;
        while (local3 < local2.length) {
            local2[local3] = ((java.lang.Integer) local1.poll()).intValue();
            local3 = local3 + 1;
        }
        return local2;
    }
}
