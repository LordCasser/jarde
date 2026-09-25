public class Lazy {
    static volatile Holder h;

    int v;

    static class Holder {
        int v;
    }

    static int get() {
        Holder local = h;
        if (local == null) {
            local = new Holder();
            h = local;
        }
        return local.v;
    }

    int viaParam(Holder p) {
        return p.v;
    }

    int viaThis() {
        return this.v;
    }
}
