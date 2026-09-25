package matrix;

public final class Runner {
    public static void main(String[] args) {
        Outer.A<String> outer = new Outer.A<>();
        System.out.println(UsePlain.make(outer, 3).getClass().getName() + ":" + Outer.A.effects);
        try {
            UsePlain.make(null, 4);
        } catch (NullPointerException expected) {
            System.out.println("plain-null:" + Outer.A.effects);
        }
        System.out.println(UseGenericObject.make(outer, 5).getClass().getName() + ":" + Outer.A.effects);
        try {
            UseGenericObject.make(null, 6);
        } catch (NullPointerException expected) {
            System.out.println("generic-object-null:" + Outer.A.effects);
        }
        System.out.println(UseGenericTyped.make(outer, 7).value() + ":" + Outer.A.effects);
        try {
            UseGenericTyped.make(null, 8);
        } catch (NullPointerException expected) {
            System.out.println("generic-typed-null:" + Outer.A.effects);
        }
        System.out.println(UsePlainRaw.make(outer, 9).getClass().getName() + ":" + Outer.A.effects);
        try {
            UsePlainRaw.make(null, 10);
        } catch (NullPointerException expected) {
            System.out.println("plain-raw-null:" + Outer.A.effects);
        }
    }
}
