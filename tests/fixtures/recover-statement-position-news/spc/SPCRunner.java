// Runs the hand-built `SPC.run()` under `-Xverify:all` and prints the trace it left: the original
// class answers `CST` (Target's class initializer, then the independent call, then the
// constructor), which is exactly the order a presentation must not move.
public final class SPCRunner {
    public static void main(String[] args) {
        Trace.value = "";
        SPC.run();
        System.out.println(Trace.value);
    }
}
