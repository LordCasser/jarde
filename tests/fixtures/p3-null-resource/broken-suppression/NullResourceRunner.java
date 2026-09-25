/** Runtime oracle for a null resource: the body runs and close does not. */
public final class NullResourceRunner {
    public static void main(String[] args) throws Exception {
        NullResourceCore.reset();
        NullResourceCore.useNullResource();
        if (NullResourceCore.bodyCalls() != 1 || NullResourceCore.closes() != 0) {
            throw new AssertionError("body=" + NullResourceCore.bodyCalls()
                    + ",closes=" + NullResourceCore.closes());
        }
        System.out.println("null-resource:bodyCalls=1,closes=0");
    }
}
