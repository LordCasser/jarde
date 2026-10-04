package demo;

public enum LabeledOps implements LabeledValue {
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

    private final String label;

    LabeledOps(String label) {
        this.label = label;
    }

    public String getLabel() {
        return label;
    }
}
