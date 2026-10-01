package defpackage;

/* JADX INFO: loaded from: hier-depth.jar:H3.class */
public class H3 {

    /* JADX INFO: loaded from: hier-depth.jar:H3$L0.class */
    static class L0 implements Sig {
        L0() {
        }

        @Override // H3.Sig
        public String tag() {
            return "l0";
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L1.class */
    static class L1 extends L0 {
        L1() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L2.class */
    static class L2 extends L1 {
        L2() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L3.class */
    static class L3 extends L2 {
        L3() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L4.class */
    static class L4 extends L3 {
        L4() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L5.class */
    static class L5 extends L4 {
        L5() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L6.class */
    static class L6 extends L5 {
        L6() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L7.class */
    static class L7 extends L6 {
        L7() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$L8.class */
    static class L8 extends L7 {
        L8() {
        }
    }

    /* JADX INFO: loaded from: hier-depth.jar:H3$Sig.class */
    interface Sig {
        String tag();
    }

    static String via(Sig sig) {
        return "d:" + sig.tag();
    }

    public static void main(String[] strArr) {
        System.out.println(via(new L7()));
        System.out.println(via(new L8()));
    }
}
