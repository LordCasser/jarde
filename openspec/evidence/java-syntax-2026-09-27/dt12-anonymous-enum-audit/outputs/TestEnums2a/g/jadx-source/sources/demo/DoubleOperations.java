package demo;

/* JADX INFO: loaded from: original.jar:demo/DoubleOperations.class */
public enum DoubleOperations implements IOps {
    TIMES("*") { // from class: demo.DoubleOperations.1
        @Override // demo.IOps
        public double apply(double x, double y) {
            return x * y;
        }
    },
    DIVIDE("/") { // from class: demo.DoubleOperations.2
        @Override // demo.IOps
        public double apply(double x, double y) {
            return x / y;
        }
    };

    private final String op;

    DoubleOperations(String op) {
        this.op = op;
    }

    public String getOp() {
        return this.op;
    }
}
