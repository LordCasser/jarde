package defpackage;

/* JADX INFO: loaded from: BoundaryTagged.class */
public final class BoundaryTagged {

    @BoundaryMark(1)
    int field;

    @BoundaryMark(2)
    int wideAndVarargs(@BoundaryMark(3) long j, @BoundaryMark(4) double d, @BoundaryMark(5) String... strArr) {
        return ((int) j) + ((int) d) + strArr.length;
    }

    @BoundaryMark(6)
    int sameTypeAtDistinctPositions(@BoundaryMark(7) int i, @BoundaryMark(8) String str) {
        return i + str.length();
    }
}
