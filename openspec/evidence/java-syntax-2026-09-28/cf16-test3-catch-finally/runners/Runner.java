package jadx.tests.integration.trycatch;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import jadx.core.dex.nodes.ClassNode;
import jadx.core.dex.visitors.IDexTreeVisitor;
import org.slf4j.LoggerFactory;

public final class Runner {
    private Runner() {
    }

    public static void main(String[] args) {
        java.lang.reflect.Method test;
        try {
            test = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls")
                    .getMethod("test", ClassNode.class, List.class);
        } catch (ReflectiveOperationException e) {
            throw new AssertionError(e);
        }
        ClassNode normal = new ClassNode();
        LoggerFactory.errorCount = 0;
        invoke(test, normal, Collections.<IDexTreeVisitor>emptyList());
        System.out.println("normal load=" + normal.loadCount + " unload=" + normal.unloadCount + " logs=" + LoggerFactory.errorCount);

        ClassNode exceptional = new ClassNode();
        LoggerFactory.errorCount = 0;
        invoke(test, exceptional, Arrays.<IDexTreeVisitor>asList(new IDexTreeVisitor() {
            @Override
            public void visit(ClassNode cls) {
                throw new IllegalStateException("visitor-failure");
            }
        }));
        System.out.println("visitor_exception load=" + exceptional.loadCount + " unload=" + exceptional.unloadCount + " logs=" + LoggerFactory.errorCount
                + " logged=" + LoggerFactory.lastError);
    }

    private static void invoke(java.lang.reflect.Method test, ClassNode cls, List<IDexTreeVisitor> passes) {
        try {
            test.invoke(null, cls, passes);
        } catch (ReflectiveOperationException e) {
            throw new AssertionError(e);
        }
    }
}
