public class MissingWriteByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public MissingWriteByteArray() {
		super();
		this.bytes = new byte[] { mark(41) };
		trace += "body:write;";
	}

	public MissingWriteByteArray(int marker) {
		super();
		trace += "body:omit:" + marker + ";";
	}
}
