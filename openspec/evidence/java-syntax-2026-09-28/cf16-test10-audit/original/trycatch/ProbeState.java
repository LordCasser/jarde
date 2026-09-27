package trycatch;

public final class ProbeState {
	static StringBuilder events;
	static String mode;

	private ProbeState() {
	}

	public static void log() {
		events.append("log;");
		if (mode.equals("logger-runtime")) {
			throw new IllegalStateException("log");
		}
	}
}
