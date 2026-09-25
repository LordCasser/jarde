public final class NullResourceExceptionalRunner {
    public static void main(String[] args) {
        RuntimeException marker = new RuntimeException("primary");
        NullResourceThrower.marker = marker;
        NullResourceThrower.bodyCalls = 0;
        NullResourceExceptionalCore.reset();
        RuntimeException actual = null;
        try {
            NullResourceExceptionalCore.useNullResourceException();
        } catch (RuntimeException failure) {
            actual = failure;
        }
        if (actual != marker || actual.getSuppressed().length != 0
                || NullResourceThrower.bodyCalls != 1
                || NullResourceExceptionalCore.closes() != 0) {
            throw new AssertionError("identity=" + (actual == marker)
                    + ",suppressed=" + (actual == null ? -1 : actual.getSuppressed().length)
                    + ",bodyCalls=" + NullResourceThrower.bodyCalls
                    + ",closes=" + NullResourceExceptionalCore.closes());
        }
        System.out.println("null-resource-exception:identity=true,suppressed=0,bodyCalls=1,closes=0");
    }
}
