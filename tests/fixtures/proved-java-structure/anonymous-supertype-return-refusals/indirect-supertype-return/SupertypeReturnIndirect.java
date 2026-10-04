interface Top {
    String tag();
}

interface Mid extends Top {
    String mark();
}

abstract class Base implements Mid {
    abstract String extra();
}

public final class SupertypeReturnIndirect {
    // Negative (recover-anonymous-supertype-return tasks 1.3b): the root method declares `Top`,
    // which `Mid` extends and `Base` implements only through `Mid`. Proving `Base <: Top` needs
    // two hierarchy walks; this slice admits one layer only (the superclass itself, its direct
    // superclass, or its directly implemented interfaces), so the shape must keep the physical
    // class text and refuse.
    static Top create() {
        return new Base() {
            @Override
            String extra() {
                return "extra";
            }

            @Override
            public String mark() {
                return "mark";
            }

            @Override
            public String tag() {
                return "tagged";
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(create().tag());
    }
}
