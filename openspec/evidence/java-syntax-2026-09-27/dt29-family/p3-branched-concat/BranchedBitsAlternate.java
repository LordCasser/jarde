package dt29p3;

public class BranchedBitsAlternate {
    public static class A {
        public boolean alpha;
        protected boolean beta;
        boolean gamma;
        private boolean delta;
        public static int reads;

        static boolean readDelta(A a) {
            reads++;
            a.alpha = false;
            a.beta = false;
            a.gamma = false;
            return a.delta;
        }
    }

    public static String bits(A a) {
        return (a.alpha ? "Y" : "N") + (a.beta ? "Y" : "N")
                + (a.gamma ? "Y" : "N") + (A.readDelta(a) ? "Y" : "N");
    }
}
