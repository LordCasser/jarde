public final class ParentEdgeDriver {
    public static void main(String[] args) {
        if (!(new ParentEdge() instanceof Runnable)) throw new AssertionError("Runnable parent missing");
        ParentEdge.test();
        System.out.println("PASS: ParentEdge implements Runnable; consume invoked");
    }
}
