package jadx.tests.integration.trycatch;

import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import jadx.core.dex.nodes.ClassNode;
import jadx.core.dex.visitors.DepthTraversal;
import jadx.core.dex.visitors.IDexTreeVisitor;

public class TestTryCatchFinally3$TestCls {
    private static final Logger LOG = LoggerFactory.getLogger(TestTryCatchFinally3$TestCls.class);

    public static void test(ClassNode cls, List<IDexTreeVisitor> passes) {
        try {
            cls.load();
            for (IDexTreeVisitor visitor : passes) {
                DepthTraversal.visit(visitor, cls);
            }
        } catch (Exception e) {
            LOG.error("Class process exception: {}", cls, e);
        } finally {
            cls.unload();
        }
    }
}
