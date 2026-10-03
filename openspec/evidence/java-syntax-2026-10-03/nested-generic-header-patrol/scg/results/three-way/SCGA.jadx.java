package defpackage;

/* JADX INFO: loaded from: SCGA.class */
public class SCGA<T extends java.lang.Comparable<T>> {
    public void note(T t) {
        java.util.Collections.singletonList(t);
    }

    public static void main(java.lang.String[] strArr) {
        new defpackage.SCGA().note("b");
        java.lang.System.out.println("note:" + "b".equals("b"));
    }
}
