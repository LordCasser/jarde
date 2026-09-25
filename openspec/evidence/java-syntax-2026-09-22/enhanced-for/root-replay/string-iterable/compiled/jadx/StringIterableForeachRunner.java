package defpackage;

import java.util.Iterator;
import java.util.NoSuchElementException;
import java.util.function.Supplier;

final class StringIterableForeachRunner {
    public static void main(String[] args) {
        ProbeIterable empty = new ProbeIterable(new String[0], null, 0);
        int emptySum = StringIterableForeach.sumLengths(empty);
        System.out.println("empty=" + emptySum + ":" + empty.counts());

        ProbeIterable many = new ProbeIterable(new String[] {"a", "bc", "de"}, null, 0);
        int manySum = StringIterableForeach.sumLengths(many);
        System.out.println("many=" + manySum + ":" + many.counts());

        try {
            StringIterableForeach.sumLengths(null);
            System.out.println("null=NORMAL");
        } catch (Throwable error) {
            System.out.println("null=" + describe(error));
        }

        ProbeIterable supplied = new ProbeIterable(new String[] {"j", "vm"}, null, 0);
        final int[] sourceCalls = {0};
        int fromSource = StringIterableForeach.sumLengthsFrom(new Supplier<Iterable<String>>() {
            @Override
            public Iterable<String> get() {
                sourceCalls[0]++;
                return supplied;
            }
        });
        System.out.println("source-many=" + fromSource + ":sourceCalls=" + sourceCalls[0]
                + ":" + supplied.counts());

        sourceCalls[0] = 0;
        try {
            StringIterableForeach.sumLengthsFrom(new Supplier<Iterable<String>>() {
                @Override
                public Iterable<String> get() {
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
            StringIterableForeach.sumLengthsFrom(new Supplier<Iterable<String>>() {
                @Override
                public Iterable<String> get() {
                    sourceCalls[0]++;
                    throw new IllegalStateException("source");
                }
            });
            System.out.println("source-throws=NORMAL:calls=" + sourceCalls[0]);
        } catch (Throwable error) {
            System.out.println("source-throws=" + describe(error) + ":calls=" + sourceCalls[0]);
        }

        ProbeIterable iteratorFailure = new ProbeIterable(new String[] {"x"}, "iterator", 1);
        run("iterator-throws", iteratorFailure);
        ProbeIterable hasNextFailure = new ProbeIterable(new String[] {"x", "yy"}, "hasNext", 2);
        run("hasNext-throws", hasNextFailure);
        ProbeIterable nextFailure = new ProbeIterable(new String[] {"x", "yy"}, "next", 2);
        run("next-throws", nextFailure);
        ProbeIterable nullElement = new ProbeIterable(new String[] {"x", null, "yy"}, null, 0);
        run("null-element", nullElement);
    }

    private static void run(String label, ProbeIterable values) {
        try {
            int sum = StringIterableForeach.sumLengths(values);
            System.out.println(label + "=" + sum + ":" + values.counts());
        } catch (Throwable error) {
            System.out.println(label + "=" + describe(error) + ":" + values.counts());
        }
    }

    private static String describe(Throwable error) {
        return error.getClass().getSimpleName() + ":" + error.getMessage();
    }

    private static final class ProbeIterable implements Iterable<String> {
        private final String[] values;
        private final String throwAt;
        private final int throwOnCall;
        private int iteratorCalls;
        private int hasNextCalls;
        private int nextCalls;

        ProbeIterable(String[] values, String throwAt, int throwOnCall) {
            this.values = values;
            this.throwAt = throwAt;
            this.throwOnCall = throwOnCall;
        }

        @Override
        public Iterator<String> iterator() {
            iteratorCalls++;
            failIf("iterator", iteratorCalls);
            return new Iterator<String>() {
                private int index;

                @Override
                public boolean hasNext() {
                    hasNextCalls++;
                    failIf("hasNext", hasNextCalls);
                    return index < values.length;
                }

                @Override
                public String next() {
                    nextCalls++;
                    failIf("next", nextCalls);
                    if (index >= values.length) {
                        throw new NoSuchElementException();
                    }
                    return values[index++];
                }

                @Override
                public void remove() {
                    throw new UnsupportedOperationException();
                }
            };
        }

        String counts() {
            return "iterator=" + iteratorCalls + ",hasNext=" + hasNextCalls + ",next=" + nextCalls;
        }

        private void failIf(String operation, int call) {
            if (operation.equals(throwAt) && call == throwOnCall) {
                throw new IllegalStateException(operation);
            }
        }
    }
}
