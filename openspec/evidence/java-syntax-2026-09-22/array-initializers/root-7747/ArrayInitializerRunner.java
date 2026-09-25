import java.util.Arrays;

public class ArrayInitializerRunner {
    private interface Case {
        Object run();
    }

    private static void run(String name, Case body) {
        ArrayInitializerProbe.reset();
        try {
            Object value = body.run();
            System.out.println(name + ":array=" + array(value) + ":trace=" + ArrayInitializerProbe.trace());
        } catch (RuntimeException error) {
            System.out.println(name + ":exception=" + error.getClass().getName() + ":message=" + error.getMessage() + ":trace=" + ArrayInitializerProbe.trace());
        }
    }

    private static String array(Object value) {
        if (value instanceof int[]) {
            return Arrays.toString((int[]) value);
        }
        if (value instanceof String[]) {
            return Arrays.toString((String[]) value);
        }
        return "-";
    }

    public static void main(String[] args) {
        run("literalInts", new Case() {
            public Object run() {
                return ArrayInitializerProbe.literalInts();
            }
        });
        run("emptyInts", new Case() {
            public Object run() {
                return ArrayInitializerProbe.emptyInts();
            }
        });
        run("literalStrings", new Case() {
            public Object run() {
                return ArrayInitializerProbe.literalStrings();
            }
        });
        run("emptyStrings", new Case() {
            public Object run() {
                return ArrayInitializerProbe.emptyStrings();
            }
        });
        for (final int mode : new int[] {-1, 0, 1, 2, 3}) {
            run("effectfulInts" + mode, new Case() {
                public Object run() {
                    return ArrayInitializerProbe.effectfulInts(mode);
                }
            });
        }
        for (final int mode : new int[] {-1, 0, 1, 2, 3}) {
            run("effectfulStrings" + mode, new Case() {
                public Object run() {
                    return ArrayInitializerProbe.effectfulStrings(mode);
                }
            });
        }
    }
}
