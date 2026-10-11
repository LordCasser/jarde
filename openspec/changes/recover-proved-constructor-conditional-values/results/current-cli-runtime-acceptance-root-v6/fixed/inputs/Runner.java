package cf11;

public final class Runner {
	private Runner() { }

	private static void run(String label, String input) {
		try {
			String result = UpstreamSequentialLoops2.asciiToLowerCase(input);
			System.out.println(label + "=" + hexUtf16(result));
		} catch (Throwable error) {
			System.out.println(label + "=" + error.getClass().getName());
		}
	}

	private static String hexUtf16(String value) {
		StringBuilder out = new StringBuilder();
		for (int i = 0; i < value.length(); i++) {
			if (i != 0) out.append(' ');
			out.append(String.format("%04X", (int) value.charAt(i)));
		}
		return out.toString();
	}

	public static void main(String[] args) {
		run("empty", "");
		run("allNonASCII", "\u0080");
		run("NUL", "\u0000");
		run("NULNUL", "\u0000\u0000");
		run("U0080NUL", "\u0080\u0000");
		run("NULU0080", "\u0000\u0080");
		run("a", "a");
		run("aNUL", "a\u0000");
		run("NULa", "\u0000a");
		run("null", null);
	}
}
