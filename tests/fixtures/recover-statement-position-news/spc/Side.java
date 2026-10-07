// The support family of the hand-built `SPC.class` (see `Generate.java`): the same observable
// effects the frozen `ordinary-new-void-effect` counterexample's family carries — a class
// initializer, an independent call and a constructor, each appending its own letter — so that the
// class-initialization order a reader can observe is `CST` for the original class.
public final class Side {
    public static void effect() {
        Trace.value += "S";
    }
}
