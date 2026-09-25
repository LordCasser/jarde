public final class NullResourceShapesRunner {
    public static void main(String[] args) throws Throwable {
        NullResourceCore.reset();
        NullResourceCore.ordinaryNullThenCatch();
        if (NullResourceCore.bodyCalls() != 1 || NullResourceCore.closes() != 0) {
            throw new AssertionError("ordinary body=" + NullResourceCore.bodyCalls()
                    + ",closes=" + NullResourceCore.closes());
        }
        NullResourceCore.reset();
        NullResourceCore.closeWithoutSuppression();
        if (NullResourceCore.bodyCalls() != 1 || NullResourceCore.closes() != 0) {
            throw new AssertionError("near TWR body=" + NullResourceCore.bodyCalls()
                    + ",closes=" + NullResourceCore.closes());
        }
        System.out.println("same-type-shapes:ordinaryCatch=1,nearTwrBody=1,closes=0");
    }
}
