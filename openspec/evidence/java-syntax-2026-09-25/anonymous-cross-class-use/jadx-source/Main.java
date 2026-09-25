package defpackage;

/* JADX INFO: loaded from: input.jar:Main.class */
public final class Main {
    public static void main(String[] strArr) {
        System.out.println("sameClass=" + (Owner.one().getClass() == Other.two().getClass()));
    }
}
