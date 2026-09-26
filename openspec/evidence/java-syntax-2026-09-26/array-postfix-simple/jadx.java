package defpackage;

/* JADX INFO: loaded from: input.jar:PostfixProbe.class */
public final class PostfixProbe {
    public static int readThenIncrement(int[] iArr, int i) {
        int i2 = iArr[i];
        iArr[i] = i2 + 1;
        return i2;
    }

    public static void main(String[] strArr) {
        int[] iArr = {41};
        System.out.println("old=" + readThenIncrement(iArr, 0) + ",new=" + iArr[0]);
    }
}
