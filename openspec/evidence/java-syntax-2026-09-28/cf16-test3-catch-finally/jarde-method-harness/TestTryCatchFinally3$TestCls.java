// Method-level harness: declaration and LOG initializer are supplied from the original class.
// The test method below is extracted unchanged from fresh Jarde class-source output.
package jadx.tests.integration.trycatch;
public class TestTryCatchFinally3$TestCls {
    private static final org.slf4j.Logger LOG = org.slf4j.LoggerFactory.getLogger(TestTryCatchFinally3$TestCls.class);
    public static void test(jadx.core.dex.nodes.ClassNode cls, java.util.List passes) {
        // @method test(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V
        // @declaration a static method of `jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            cls.load();
            java.util.Iterator e = passes.iterator();
            while (e.hasNext()) {
                jadx.core.dex.visitors.IDexTreeVisitor visitor = (jadx.core.dex.visitors.IDexTreeVisitor) e.next();
                jadx.core.dex.visitors.DepthTraversal.visit(visitor, cls);
            }
        } catch (java.lang.Exception e) {
            jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls.LOG.error("Class process exception: {}", (java.lang.Object) cls, (java.lang.Object) e);
        } finally {
            cls.unload();
        }
        return;
    }
}
