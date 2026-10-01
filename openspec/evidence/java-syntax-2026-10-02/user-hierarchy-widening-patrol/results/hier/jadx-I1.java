package defpackage;

/* JADX INFO: loaded from: fam.jar:I1.class */
public class I1 {

    /* JADX INFO: loaded from: fam.jar:I1$En.class */
    static class En implements Greet {
        En() {
        }

        @Override // I1.Greet
        public String name() {
            return "en";
        }

        @Override // I1.Greet
        public String hello(String str) {
            return "hello:" + str;
        }
    }

    /* JADX INFO: loaded from: fam.jar:I1$Greet.class */
    interface Greet {
        default String hello(String str) {
            return "hi:" + str;
        }

        static Greet of() {
            return new Greet() { // from class: I1.Greet.1
                @Override // I1.Greet
                public String name() {
                    return "static";
                }
            };
        }

        String name();
    }

    public static String useDefault() {
        return new Greet() { // from class: I1.1
            @Override // I1.Greet
            public String name() {
                return "anon";
            }
        }.hello("d");
    }

    public static String useOverride() {
        return new En().hello("d");
    }

    public static String useStatic() {
        return Greet.of().name();
    }

    public static String viaInterface(Greet greet, String str) {
        return greet.hello(str);
    }

    public static void main(String[] strArr) {
        System.out.println(useDefault());
        System.out.println(useOverride());
        System.out.println(useStatic());
        System.out.println(viaInterface(new En(), "v"));
    }
}
