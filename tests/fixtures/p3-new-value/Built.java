public class Built {
    Object a;
    static Object s;

    Built() {
        this.a = new Object();
    }

    void set() {
        this.a = new Object();
    }

    static void setStatic() {
        Built.s = new Object();
    }

    Object localNew() {
        Object o = new Object();
        return o;
    }

    Object directNew() {
        return new Object();
    }

    String take(Object o) {
        return String.valueOf(o);
    }

    String argNew() {
        return take(new Object());
    }
}
