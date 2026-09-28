package org.slf4j;

public final class LoggerFactory {
    public static int errorCount;
    public static Object lastError;
    private static final Logger LOGGER = new Logger() {
        @Override
        public void error(String format, Object first, Object second) {
            errorCount++;
            lastError = second;
        }
    };

    private LoggerFactory() {
    }

    public static Logger getLogger(Class<?> cls) {
        return LOGGER;
    }
}
