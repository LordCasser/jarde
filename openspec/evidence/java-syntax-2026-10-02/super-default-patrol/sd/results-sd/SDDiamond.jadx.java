package defpackage;

/* JADX WARN: Classes with same name are omitted, all sources:
  sd-negatives-orig.jar:SDDiamond.class
  sd-variants.jar:SDDiamond.class
 */
/* JADX INFO: loaded from: sd-variants.jar:SDDiamond.class */
public class SDDiamond {

    /* JADX WARN: Classes with same name are omitted, all sources:
      sd-negatives-orig.jar:SDDiamond$A.class
      sd-variants.jar:SDDiamond$A.class
     */
    /* JADX INFO: loaded from: sd-variants.jar:SDDiamond$A.class */
    interface A {
        default String name() {
            return "A";
        }

        default void log(String str) {
            System.out.println("log:" + str);
        }

        default String greet(int i) {
            return "g" + i;
        }
    }

    /* JADX WARN: Classes with same name are omitted, all sources:
      sd-negatives-orig.jar:SDDiamond$B.class
      sd-variants.jar:SDDiamond$B.class
     */
    /* JADX INFO: loaded from: sd-variants.jar:SDDiamond$B.class */
    interface B {
        default String name() {
            return "B";
        }
    }

    /* JADX WARN: Classes with same name are omitted, all sources:
      sd-negatives-orig.jar:SDDiamond$Use.class
      sd-variants.jar:SDDiamond$Use.class
     */
    /* JADX INFO: loaded from: sd-variants.jar:SDDiamond$Use.class */
    static class Use implements A, B {
        Use() {
        }

        @Override // SDDiamond.A, SDDiamond.B
        public String name() {
            return super.name() + super.name();
        }

        String mixed() {
            return super.name() + "!";
        }

        void logCall() {
            super.log("hi");
        }

        String greetCall() {
            return super.greet(7);
        }
    }

    public static void main(String[] strArr) {
        Use use = new Use();
        System.out.println(use.name());
        System.out.println(use.mixed());
        use.logCall();
        System.out.println(use.greetCall());
    }
}
