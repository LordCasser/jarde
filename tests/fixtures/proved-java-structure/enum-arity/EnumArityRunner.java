package probe;

import java.util.Arrays;

public final class EnumArityRunner {
	public static void main(String[] args) {
		System.out.println("empty=" + Empty.values().length);
		System.out.println("one=" + One.ONLY.name() + ":" + One.ONLY.ordinal() + "/" + One.values().length);
		System.out.println("four=" + Arrays.toString(Four.values()));
	}
}
