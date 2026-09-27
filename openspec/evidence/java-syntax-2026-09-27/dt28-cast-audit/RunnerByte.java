package dt28;

public final class RunnerByte {
	public static void main(String[] args) {
		System.out.println(ByteConditionalCall.run(9L, true) + ":"
				+ ByteConditionalCall.run(9L, false));
	}
}
