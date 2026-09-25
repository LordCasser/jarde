package matrix;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/Runner.class */
public final class Runner {
    public static void main(String[] strArr) {
        Outer.A a = new Outer.A();
        System.out.println(UsePlain.make(a, 3).getClass().getName() + ":" + Outer.A.effects);
        try {
            UsePlain.make(null, 4);
        } catch (NullPointerException e) {
            System.out.println("plain-null:" + Outer.A.effects);
        }
        System.out.println(UseGenericObject.make(a, 5).getClass().getName() + ":" + Outer.A.effects);
        try {
            UseGenericObject.make(null, 6);
        } catch (NullPointerException e2) {
            System.out.println("generic-object-null:" + Outer.A.effects);
        }
        System.out.println(UseGenericTyped.make(a, 7).value() + ":" + Outer.A.effects);
        try {
            UseGenericTyped.make(null, 8);
        } catch (NullPointerException e3) {
            System.out.println("generic-typed-null:" + Outer.A.effects);
        }
        System.out.println(UsePlainRaw.make(a, 9).getClass().getName() + ":" + Outer.A.effects);
        try {
            UsePlainRaw.make(null, 10);
        } catch (NullPointerException e4) {
            System.out.println("plain-raw-null:" + Outer.A.effects);
        }
    }
}
