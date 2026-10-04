interface Renderer {
    String render();
}

abstract class Base implements Renderer {
    public abstract String render();
}

public final class ParameterizedSupertypeReturn {
    // Negative (recover-anonymous-parameterized-root tasks 1.3, ring 2 boundary): the root
    // method declares the supertype `Renderer` while the allocation's direct superclass is
    // `Base`. The return part of the descriptor must stay exactly the superclass type — this
    // slice widens only the parameter table, never the return type.
    private static Renderer create(final String captured) {
        return new Base() {
            @Override
            public String render() {
                return "r:" + captured;
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(create("captured-value").render());
    }
}
