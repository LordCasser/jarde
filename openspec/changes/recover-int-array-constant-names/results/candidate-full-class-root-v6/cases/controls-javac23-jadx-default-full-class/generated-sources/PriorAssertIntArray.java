package defpackage;

/* JADX INFO: loaded from: IntArrayConstantTargets.jar:PriorAssertIntArray.class */
public class PriorAssertIntArray {
    static final int VALUE = 7;
    static final /* synthetic */ boolean $assertionsDisabled;

    static {
        $assertionsDisabled = !PriorAssertIntArray.class.desiredAssertionStatus();
    }

    static int[] value(boolean ok) {
        if ($assertionsDisabled || ok) {
            return new int[]{VALUE};
        }
        throw new AssertionError();
    }
}
