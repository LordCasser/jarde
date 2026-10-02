package defpackage;

/* JADX INFO: loaded from: vn4.jar:VN4.class */
public class VN4 {
    static int area(Other.Box box) {
        return box.size(12);
    }

    static boolean marked(Object obj) {
        return obj instanceof Other.Mark;
    }

    public static void main(String[] strArr) {
        Other.Tagged tagged = new Other.Tagged();
        System.out.println(area(tagged) + ":" + marked(tagged) + ":" + tagged.tag());
    }
}
