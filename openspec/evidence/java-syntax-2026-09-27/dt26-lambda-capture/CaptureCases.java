package dt26;

public class CaptureCases {
    public java.util.function.IntUnaryOperator add(int base) {
        return x -> x + base;
    }

    public java.util.function.IntSupplier bound(int delta) {
        return () -> this.number() + delta;
    }

    public int number() {
        return -3;
    }
}
