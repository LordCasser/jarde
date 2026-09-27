package demo;

/* JADX INFO: loaded from: original.jar:demo/DoubleOperations.class */
public enum DoubleOperations implements IOps {
    TIMES("*") { // from class: demo.DoubleOperations.1
        @Override // demo.IOps
        public double apply(double d, double d2) {
            return d * d2;
        }
    },
    DIVIDE("/") { // from class: demo.DoubleOperations.2
        @Override // demo.IOps
        public double apply(double d, double d2) {
            return d / d2;
        }
    };

    private final String op;

    DoubleOperations(String str) {
        this.op = str;
    }

    public String getOp() {
        return this.op;
    }
}
