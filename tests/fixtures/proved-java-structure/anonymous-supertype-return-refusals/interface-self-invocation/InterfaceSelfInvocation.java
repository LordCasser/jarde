interface Renderer {
    String tag();
}

public final class InterfaceSelfInvocation {
    // Containment probe (recover-anonymous-supertype-return, root ruling's mandatory item 1):
    // the anonymous body invokes its own `tag()` on `this`, so javac emits
    // `invokevirtual InterfaceSelfInvocation$1.tag:()Ljava/lang/String;` — a method symbol
    // whose owner is the anonymous class itself, inside its own body. The interface anonymous
    // path shares the owner census that refuses such uses
    // (`anonymous_interface_child_additional_use`); the slice's new census arm is gated on a
    // discriminant the interface path does not pass, so this fixture must render identically
    // before and after the slice — the direct path may not open the interface path's acceptance
    // set as a side effect (the `recover-anonymous-local-decl-site` criterion 5 pattern).
    static Renderer create() {
        return new Renderer() {
            @Override
            public String tag() {
                return "tagged";
            }

            @Override
            public String toString() {
                return tag();
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(create());
    }
}
