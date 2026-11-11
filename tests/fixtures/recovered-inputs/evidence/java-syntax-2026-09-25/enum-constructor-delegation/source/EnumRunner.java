import java.lang.reflect.Constructor;

public final class EnumRunner {
    private EnumRunner() {
    }

    public static void main(String[] args) {
        DelegatingEnum[] values = DelegatingEnum.values();
        check(values.length == 2, "values length");
        check(values[0] == DelegatingEnum.ZERO, "ZERO identity/order");
        check(values[1] == DelegatingEnum.ONE, "ONE identity/order");
        check(DelegatingEnum.ZERO.name().equals("ZERO"), "ZERO name");
        check(DelegatingEnum.ONE.name().equals("ONE"), "ONE name");
        check(DelegatingEnum.ZERO.ordinal() == 0, "ZERO ordinal");
        check(DelegatingEnum.ONE.ordinal() == 1, "ONE ordinal");
        check(DelegatingEnum.ZERO.value() == 0, "ZERO field");
        check(DelegatingEnum.ONE.value() == 1, "ONE field");
        check(ConstructorEffects.calls() == 2, "constructor call count");
        check(ConstructorEffects.events().equals("0,1"), "constructor event order");
        Constructor<?>[] constructors = DelegatingEnum.class.getDeclaredConstructors();
        check(constructors.length == 2, "declared constructor count");
        int firstCount = constructors[0].getParameterTypes().length;
        int secondCount = constructors[1].getParameterTypes().length;
        int smaller = Math.min(firstCount, secondCount);
        int larger = Math.max(firstCount, secondCount);
        check(smaller == 2 && larger == 3, "declared constructor parameter counts");
        System.out.println("values=ZERO:0,ONE:1");
        System.out.println("effects=" + ConstructorEffects.calls() + ":" + ConstructorEffects.events());
        System.out.println("declared-constructors=" + smaller + "," + larger);
    }

    private static void check(boolean condition, String label) {
        if (!condition) {
            throw new AssertionError(label);
        }
    }
}
