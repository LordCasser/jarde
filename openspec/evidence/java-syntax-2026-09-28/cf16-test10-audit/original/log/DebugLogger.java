package log;

import java.io.IOException;

public final class DebugLogger {
	public enum LogLevel {
		ERROR
	}

	public void logException(LogLevel level, IOException exception) {
		trycatch.ProbeState.log();
	}
}
