public final class BoundaryRunner {
    public static void main(String[] args) {
        System.out.println(BoundaryProbe.third(true, false));
        System.out.println(BoundaryProbe.third(false, true));
        System.out.println(BoundaryProbe.third(false, false));
        System.out.println(BoundaryProbe.backedge(2));
        System.out.println(BoundaryProbe.backedge(0));
        System.out.println(BoundaryProbe.exception("x"));
        System.out.println(BoundaryProbe.exception(null));
    }
}
