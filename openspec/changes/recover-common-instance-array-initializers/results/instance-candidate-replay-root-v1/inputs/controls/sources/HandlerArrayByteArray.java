public class HandlerArrayByteArray {
	static String trace = "";
	static boolean fail;
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		if (fail) {
			throw new IllegalStateException("requested failure");
		}
		return (byte) value;
	}

	public HandlerArrayByteArray() {
		super();
		try {
			this.bytes = new byte[] { mark(91) };
		} catch (IllegalStateException ex) {
			trace += "caught;";
		}
		trace += "body:noarg;";
	}

	public HandlerArrayByteArray(int marker) {
		super();
		try {
			this.bytes = new byte[] { mark(91) };
		} catch (IllegalStateException ex) {
			trace += "caught;";
		}
		trace += "body:int:" + marker + ";";
	}
}
