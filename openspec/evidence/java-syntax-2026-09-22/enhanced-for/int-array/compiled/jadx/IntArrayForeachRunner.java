package defpackage;

import java.util.function.Supplier;

final class IntArrayForeachRunner {
    public static void main(String[] args) {
        System.out.println("empty=" + IntArrayForeach.sum(new int[0]));
        System.out.println("many=" + IntArrayForeach.sum(new int[] {-2, 3, 5}));

        try {
            IntArrayForeach.sum(null);
            System.out.println("null=NORMAL");
        } catch (Throwable error) {
            System.out.println("null=" + describe(error));
        }

        final int[] sourceCalls = {0};
        int fromSource = IntArrayForeach.sumFrom(new Supplier<int[]>() {
            @Override
            public int[] get() {
                sourceCalls[0]++;
                return new int[] {4, 7, 9};
            }
        });
        System.out.println("source-many=" + fromSource + ":calls=" + sourceCalls[0]);

        sourceCalls[0] = 0;
        try {
            IntArrayForeach.sumFrom(new Supplier<int[]>() {
                @Override
                public int[] get() {
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
            IntArrayForeach.sumFrom(new Supplier<int[]>() {
                @Override
                public int[] get() {
                    sourceCalls[0]++;
                    throw new IllegalStateException("source");
                }
            });
            System.out.println("source-throws=NORMAL:calls=" + sourceCalls[0]);
        } catch (Throwable error) {
            System.out.println("source-throws=" + describe(error) + ":calls=" + sourceCalls[0]);
        }
    }

    private static String describe(Throwable error) {
        return error.getClass().getSimpleName() + ":" + error.getMessage();
    }
}
