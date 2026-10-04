package defpackage;

/* JADX INFO: loaded from: AN.class */
public class AN {
    void plain() {
        java.lang.System.out.println("plain");
    }

    AN.Base mkBase() {
        return new AN.1(this);
    }

    AN.Greeter mkGreet() {
        return new AN.2(this);
    }

    AN.Greeter cap(int i) {
        return new AN.3(this, i);
    }

    public static void main(java.lang.String[] strArr) {
        defpackage.AN an = new defpackage.AN();
        an.plain();
        an.mkBase().hi();
        an.mkGreet().greet();
        an.cap(7).greet();
    }
}
