/** Source-only runner; each invocation is a fresh JVM so both <clinit> branches execute. */
public final class FinalStaticRunner {
    private FinalStaticRunner() {}

    public static void main(String[] args) {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected true or false");
        }
        System.setProperty("jarde.final-static.branch", args[0]);
        System.out.println(FinalStaticProbe.snapshot());
        System.out.println(FinalStaticSupport.order());
        System.out.println("read=" + FinalStaticProbe.readAfterAssign());
        System.out.println("instance=" + new FinalStaticProbe(41).instanceValue());
    }
}
