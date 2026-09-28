package org.slf4j;

public final class LoggerFactory {
    public static int errorCount;
    public static Object lastError;
    public static RuntimeException failure;
    private static final Logger LOGGER = new Logger() {
        @Override
        public void error(String format, Object first, Object second) {
            jadx.core.dex.nodes.ClassNode.event("logger");
            errorCount++;
            lastError = second;
            if (failure != null) {
                throw failure;
            }
        }
    };

    private LoggerFactory() {
    }

    public static Logger getLogger(Class<?> cls) {
        return LOGGER;
    }
}
