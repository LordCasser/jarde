package defpackage;

/* JADX INFO: loaded from: input.jar:Inner.class */
public class Inner {
    static Object observed;
    int f = 37;

    Runnable make() {
        return new Runnable() { // from class: Inner.1
            @Override // java.lang.Runnable
            public void run() {
                Inner.observed = Inner.this;
                Inner.this.f++;
            }
        };
    }

    public static void main(String[] strArr) {
        Inner inner = new Inner();
        inner.make().run();
        System.out.println(observed == inner);
        System.out.println(inner.f);
    }
}
