public final class MixedInstanceControls {
    static boolean bValue;
    static boolean cValue;
    static Box saved;
    static DerivedBox lastDerived;

    static final class Box {
        boolean result;
        int number;
    }

    static class BaseBox {
        boolean result;
    }

    static final class DerivedBox extends BaseBox {
    }

    static Box target(boolean ignored) { return new Box(); }
    static DerivedBox derivedTarget(boolean ignored) {
        lastDerived = new DerivedBox();
        return lastDerived;
    }
    static boolean b() { return bValue; }
    static boolean c() { return cValue; }

    static void numeric(boolean a, boolean select) {
        target(select).number = (a && b()) || c() ? 1 : 0;
    }

    static boolean duplicated(boolean a, boolean select) {
        return target(select).result = (a && b()) || c();
    }

    static void compound(boolean a, boolean select) {
        target(select).result |= (a && b()) || c();
    }

    static void sharedReceiver(boolean a, boolean select) {
        (saved = target(select)).result = (a && b()) || c();
    }

    static void protectedWrite(boolean a, boolean select) {
        try {
            target(select).result = (a && b()) || c();
        } catch (RuntimeException ignored) {
        }
    }

    static void inheritedOwner(boolean a, boolean select) {
        ((BaseBox) derivedTarget(select)).result = (a && b()) || c();
    }
}
