package discardprobe;

/** Independent deterministic runtime oracle for the complete source-probe class. */
public final class DiscardedCallSourceProbeRunner {
    private DiscardedCallSourceProbeRunner() {}

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    public static void main(String[] args) {
        DiscardedCallSourceProbe.discardStatic(false);
        System.out.println("static-normal=completed");
        try {
            DiscardedCallSourceProbe.discardStatic(true);
            throw new AssertionError("discardStatic(true) returned");
        } catch (IllegalStateException expected) {
            require("give-failed".equals(expected.getMessage()), "throwing call changed");
            System.out.println("static-throw=give-failed");
        }
        require("appended".equals(DiscardedCallSourceProbe.discardAppend("appended")),
                "append statement changed");
        System.out.println("append=appended");
        require("listed".equals(DiscardedCallSourceProbe.discardListAdd("listed")),
                "List.add statement changed");
        System.out.println("list-add=listed");
        require("given".equals(DiscardedCallSourceProbe.consumeReturn()),
                "return-consumed call changed");
        System.out.println("return-consumed=given");
        require("given".equals(DiscardedCallSourceProbe.deferToLocal()),
                "local-deferred call changed");
        System.out.println("local-deferred=given");
    }
}
