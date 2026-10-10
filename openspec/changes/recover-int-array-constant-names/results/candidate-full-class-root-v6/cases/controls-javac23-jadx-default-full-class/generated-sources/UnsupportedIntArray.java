package defpackage;

/* JADX INFO: loaded from: IntArrayConstantTargets.jar:UnsupportedIntArray.class */
public class UnsupportedIntArray {
    static final int VALUE = 7;

    static int make() {
        return VALUE;
    }

    static int[][] nested() {
        return new int[][]{new int[]{VALUE}};
    }

    static int[] unsupportedLeaves(long source, int offset) {
        return new int[]{make(), (int) source, offset + VALUE};
    }
}
