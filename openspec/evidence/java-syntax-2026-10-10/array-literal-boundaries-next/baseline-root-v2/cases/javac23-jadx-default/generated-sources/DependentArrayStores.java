package defpackage;

/* JADX INFO: loaded from: ArrayLiteralBoundaryTargets.jar:DependentArrayStores.class */
public class DependentArrayStores {
    public int[] test() {
        int[] iArr = new int[3];
        iArr[0] = 1;
        iArr[1] = iArr[0] + 1;
        iArr[2] = iArr[1] + 1;
        return iArr;
    }
}
