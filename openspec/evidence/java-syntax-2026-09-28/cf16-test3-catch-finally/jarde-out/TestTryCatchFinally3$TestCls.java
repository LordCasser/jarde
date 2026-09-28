// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally3$TestCls extends java.lang.Object {
    private static final org.slf4j.Logger LOG;

    public TestTryCatchFinally3$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
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

    static {
        // jarde: not recovered: the recovery run for `<clinit>()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method <clinit>()V
        // @declaration a static initializer of `jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 is not part of the provable subset
        // @bytecode 2
        // the dependency chain from BCI 2 to final consumer 5 is not bounded
        // @bytecode 5 2
        // the value at BCI 5 was produced by a saved declaration this run could not commit
    }
}
