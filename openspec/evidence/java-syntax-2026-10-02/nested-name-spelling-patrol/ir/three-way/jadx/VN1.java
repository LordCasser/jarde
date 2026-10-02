package defpackage;

/* JADX INFO: loaded from: vn1.jar:VN1.class */
public class VN1 {
    static Other.Inner box = new Other.Inner();

    static int use(Other.Inner inner, int i) {
        Other.Inner inner2 = new Other.Inner();
        boolean z = inner2 instanceof Other.Inner;
        int iId = inner2.id(i);
        int iTwice = Other.Inner.twice(i);
        if (z) {
            return inner.id(i) + iId;
        }
        return iTwice;
    }

    public static void main(String[] strArr) {
        System.out.println(use(new Other.Inner(), 21));
    }
}
