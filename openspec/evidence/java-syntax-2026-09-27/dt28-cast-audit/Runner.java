package dt28;

public final class Runner {
	public static void main(String[] args) {
		System.out.println(PrimitiveCasts.widenChar((char) 22));
		System.out.println(PrimitiveCasts.truncateLong((1L << 32) + 8));
		System.out.println(PrimitiveCasts.narrowLong(258L) + ":"
				+ PrimitiveCasts.narrowInt(65538) + ":"
				+ (int) PrimitiveCasts.narrowChar(65538));
		System.out.println(PrimitiveCasts.byteConditional(true) + ":"
				+ PrimitiveCasts.byteConditional(false));
		System.out.println(PrimitiveCasts.promotedConditional(true, 3, 4L) + ":"
				+ PrimitiveCasts.promotedConditional(false, 3, 4L));
		System.out.println("ok");
	}
}
