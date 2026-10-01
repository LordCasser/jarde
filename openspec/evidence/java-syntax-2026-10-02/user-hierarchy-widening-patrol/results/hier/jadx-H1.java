package defpackage;

/* JADX INFO: loaded from: hier-fam.jar:H1.class */
public class H1 {

    /* JADX INFO: loaded from: hier-fam.jar:H1$Caller.class */
    static class Caller {
        Caller() {
        }

        String call(Greet greet) {
            return "call:" + greet.name();
        }
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Fin.class */
    static final class Fin {
        Fin() {
        }

        public String name() {
            return "fin";
        }
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Greet.class */
    interface Greet {
        default String hello(String str) {
            return "hi:" + str;
        }

        String name();
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Mid.class */
    static class Mid implements Greet {
        Mid() {
        }

        @Override // H1.Greet
        public String name() {
            return "mid";
        }
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Multi.class */
    static class Multi implements Greet, Other {
        Multi() {
        }

        @Override // H1.Greet
        public String name() {
            return "multi";
        }

        @Override // H1.Other
        public String tag() {
            return "mtag";
        }
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Other.class */
    interface Other {
        String tag();
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$Sub.class */
    interface Sub extends Greet {
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$TwoLevel.class */
    static class TwoLevel extends Mid {
        TwoLevel() {
        }
    }

    /* JADX INFO: loaded from: hier-fam.jar:H1$ViaSub.class */
    static class ViaSub implements Sub {
        ViaSub() {
        }

        @Override // H1.Greet
        public String name() {
            return "sub";
        }
    }

    static String via(Greet greet, String str) {
        return greet.hello(str);
    }

    static String viaOther(Other other) {
        return "tag:" + other.tag();
    }

    static String lead(String str, Greet greet) {
        return str + ":" + greet.hello("x");
    }

    static String viaObject(Object obj) {
        return "obj";
    }

    public static void main(String[] strArr) {
        System.out.println(via(new TwoLevel(), "two"));
        System.out.println(via(new Multi(), "multi"));
        System.out.println(viaOther(new Multi()));
        System.out.println(via(new Greet() { // from class: H1.1
            @Override // H1.Greet
            public String name() {
                return "anon";
            }
        }, "anon"));
        System.out.println(via(new ViaSub(), "sub"));
        System.out.println(lead("lead", new Multi()));
        System.out.println(new Caller().call(new TwoLevel()));
        System.out.println(viaObject(new Fin()));
    }
}
