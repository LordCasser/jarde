package demo;

public enum C1 implements demo.IOps {
    TIMES("*") {
        public double apply(double arg1, double arg3) {
            return arg1 * arg3;
        }
    },
    DIVIDE("/") {
        public double apply(double arg1, double arg3) {
            return arg1 / arg3;
        }
    };

    private final java.lang.String op;

    private C1(java.lang.String arg0) {
        this.op = arg0;
    }

    public java.lang.String getOp() {
        return this.op;
    }
}
