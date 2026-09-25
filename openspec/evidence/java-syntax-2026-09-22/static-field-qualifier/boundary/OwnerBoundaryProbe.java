public final class OwnerBoundaryProbe {
    static Child receiver() { return null; }
    public static int call() { return receiver().ping(); }
}
