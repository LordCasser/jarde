import java.util.function.Supplier;

final class ObjectArrayForeachRunner {
    public static void main(String[] args) {
        System.out.println("empty=" + ObjectArrayForeach.sumHash(new Object[0]));

        Probe first = new Probe(2, false);
        Probe last = new Probe(5, false);
        int many = ObjectArrayForeach.sumHash(new Object[] {first, last});
        System.out.println("many=" + many + ":hashCalls=" + first.calls + "," + last.calls);

        try {
            ObjectArrayForeach.sumHash(null);
            System.out.println("null=NORMAL");
        } catch (Throwable error) {
            System.out.println("null=" + describe(error));
        }

        final int[] sourceCalls = {0};
        Probe supplied = new Probe(11, false);
        int fromSource = ObjectArrayForeach.sumHashFrom(new Supplier<Object[]>() {
            @Override
            public Object[] get() {
                sourceCalls[0]++;
                return new Object[] {supplied};
            }
        });
        System.out.println("source-many=" + fromSource + ":calls=" + sourceCalls[0]
                + ":hashCalls=" + supplied.calls);

        sourceCalls[0] = 0;
        try {
            ObjectArrayForeach.sumHashFrom(new Supplier<Object[]>() {
                @Override
                public Object[] get() {
                    sourceCalls[0]++;
                    return null;
                }
            });
            System.out.println("source-null=NORMAL:calls=" + sourceCalls[0]);
        } catch (Throwable error) {
            System.out.println("source-null=" + describe(error) + ":calls=" + sourceCalls[0]);
        }

        sourceCalls[0] = 0;
        try {
            ObjectArrayForeach.sumHashFrom(new Supplier<Object[]>() {
                @Override
                public Object[] get() {
                    sourceCalls[0]++;
                    throw new IllegalStateException("source");
                }
            });
            System.out.println("source-throws=NORMAL:calls=" + sourceCalls[0]);
        } catch (Throwable error) {
            System.out.println("source-throws=" + describe(error) + ":calls=" + sourceCalls[0]);
        }

        Probe beforeFailure = new Probe(3, false);
        Probe failure = new Probe(0, true);
        Probe afterFailure = new Probe(13, false);
        try {
            ObjectArrayForeach.sumHash(new Object[] {beforeFailure, failure, afterFailure});
            System.out.println("body-throws=NORMAL");
        } catch (Throwable error) {
            System.out.println("body-throws=" + describe(error) + ":hashCalls="
                    + beforeFailure.calls + "," + failure.calls + "," + afterFailure.calls);
        }
    }

    private static String describe(Throwable error) {
        return error.getClass().getSimpleName() + ":" + error.getMessage();
    }

    private static final class Probe {
        private final int result;
        private final boolean fail;
        private int calls;

        Probe(int result, boolean fail) {
            this.result = result;
            this.fail = fail;
        }

        @Override
        public int hashCode() {
            calls++;
            if (fail) {
                throw new IllegalStateException("hash");
            }
            return result;
        }
    }
}
