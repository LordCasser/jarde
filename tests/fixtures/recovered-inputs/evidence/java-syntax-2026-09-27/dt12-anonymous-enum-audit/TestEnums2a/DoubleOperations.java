package demo;

public enum DoubleOperations implements IOps {
    TIMES("*") {
        @Override
        public double apply(double x, double y) {
            return x * y;
        }
    },
    DIVIDE("/") {
        @Override
        public double apply(double x, double y) {
            return x / y;
        }
    };

    private final String op;

    DoubleOperations(String op) {
        this.op = op;
    }

    public String getOp() {
        return op;
    }
}
