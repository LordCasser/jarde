package pkg;

/* JADX INFO: loaded from: sd-variants.jar:pkg/SDPacked.class */
public class SDPacked {

    /* JADX INFO: loaded from: sd-variants.jar:pkg/SDPacked$I.class */
    public interface I {
        default String name() {
            return "PI";
        }
    }

    /* JADX INFO: loaded from: sd-variants.jar:pkg/SDPacked$Use.class */
    public static class Use implements I {
        @Override // pkg.SDPacked.I
        public String name() {
            return super.name();
        }
    }

    public static void main(String[] strArr) {
        System.out.println(new Use().name());
    }
}
