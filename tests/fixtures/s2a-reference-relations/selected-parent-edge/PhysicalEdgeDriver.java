public final class PhysicalEdgeDriver {
    public static void main(String[] args) {
        if (!(new PhysicalEdge() instanceof EdgeTarget)) throw new AssertionError("EdgeTarget parent missing");
        PhysicalEdge.test();
        System.out.println("PASS: PhysicalEdge implements EdgeTarget; consume invoked");
    }
}
