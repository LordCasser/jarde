package jadx.core.dex.visitors;

public final class DepthTraversal {
    private DepthTraversal() {
    }

    public static void visit(IDexTreeVisitor visitor, jadx.core.dex.nodes.ClassNode cls) {
        visitor.visit(cls);
    }
}
