package jadx.tests.integration.trycatch;

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
        for (String mode : new String[] {"normal", "load_exception", "load_error", "visitor_exception",
                "visitor_error", "logger_exception", "unload_normal", "unload_catch", "unload_handler"}) {
            run(test, mode);
        }
    }

    private static void run(java.lang.reflect.Method test, String mode) {
        ClassNode cls = new ClassNode();
        ClassNode.events.setLength(0);
        LoggerFactory.errorCount = 0;
        LoggerFactory.lastError = null;
        LoggerFactory.failure = null;
        Throwable initial = mode.endsWith("error") ? new AssertionError("initial") : new IllegalStateException("initial");
        RuntimeException loggerFailure = new IllegalArgumentException("logger-failure");
        RuntimeException unloadFailure = new IllegalArgumentException("unload-failure");
        if (mode.startsWith("load")) {
            cls.loadFailure = initial;
        }
        if (mode.startsWith("unload")) {
            cls.unloadFailure = unloadFailure;
        }
        if (mode.equals("logger_exception")) {
            LoggerFactory.failure = loggerFailure;
        }
        List<IDexTreeVisitor> passes = Collections.emptyList();
        if (mode.startsWith("visitor") || mode.equals("logger_exception") || mode.equals("unload_catch")
                || mode.equals("unload_handler")) {
            passes = Collections.<IDexTreeVisitor>singletonList(new IDexTreeVisitor() {
                @Override
                public void visit(ClassNode ignored) {
                    ClassNode.event("visitor");
                    if (!mode.equals("unload_handler")) {
                        throwFailure(initial);
                    }
                    throw new AssertionError("handler-failure");
                }
            });
        }
        if (mode.equals("unload_handler")) {
            LoggerFactory.failure = loggerFailure;
        }
        Throwable terminal = null;
        try {
            test.invoke(null, cls, passes);
        } catch (java.lang.reflect.InvocationTargetException e) {
            terminal = e.getCause();
        } catch (IllegalAccessException e) {
            throw new AssertionError(e);
        }
        String result = terminal == null ? "ok" : terminal.getClass().getSimpleName() + ":" + terminal.getMessage();
        String identity = terminal == initial ? "initial" : terminal == loggerFailure ? "logger"
                : terminal == unloadFailure ? "unload" : "other";
        System.out.println(mode + " events=" + ClassNode.events + " load=" + cls.loadCount + " unload=" + cls.unloadCount
                + " logs=" + LoggerFactory.errorCount + " loggedOriginal=" + (LoggerFactory.lastError == initial)
                + " terminal=" + result + " identity=" + identity);
    }

    private static void throwFailure(Throwable failure) {
        if (failure instanceof Error) {
            throw (Error) failure;
        }
        throw (RuntimeException) failure;
    }
}
