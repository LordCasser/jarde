public class ThisDelegatingByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public ThisDelegatingByteArray() {
		this(7);
		trace += "body:delegate;";
	}

	public ThisDelegatingByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(21), mark(22) };
		trace += "body:target:" + marker + ";";
	}
}
