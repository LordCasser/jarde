/**
 * Java 8 source for the permanent FinalStaticProbe class-file fixture.
 *
 * The source-only helper supplies observable initialization order.  The generated source has to
 * keep the static initializer in place: the two calls, the two arms of branchValue, and the read
 * of first/second/branchValue are all part of one <clinit> body.
 */
public class FinalStaticProbe {
    public static final int first;
    public static final int second;
    public static final int branchValue;
    public static final int local0;
    public static final int local0_2;
    public static final int afterAssign;
    public static final int constantValue = 17;

    private final int instanceValue;

    static {
        first = FinalStaticSupport.next("first");
        second = FinalStaticSupport.next("second");

        int seed = FinalStaticSupport.next("local");
        local0 = seed;
        local0_2 = FinalStaticSupport.next("local0_2");

        if (FinalStaticSupport.branch()) {
            branchValue = FinalStaticSupport.next("branch-true");
        } else {
            branchValue = FinalStaticSupport.next("branch-false");
        }

        afterAssign = first + second + branchValue;
    }

    public FinalStaticProbe(int value) {
        instanceValue = value;
    }

    public int instanceValue() {
        return instanceValue;
    }

    public static int readAfterAssign() {
        return afterAssign;
    }

    public static String snapshot() {
        return first + ":" + second + ":" + branchValue + ":" + local0 + ":" + local0_2
                + ":" + afterAssign + ":" + constantValue;
    }
}
