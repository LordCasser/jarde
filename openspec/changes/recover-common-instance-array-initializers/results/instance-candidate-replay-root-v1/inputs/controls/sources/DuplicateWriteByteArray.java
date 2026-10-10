public class DuplicateWriteByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public DuplicateWriteByteArray() {
		super();
		this.bytes = new byte[] { mark(51) };
		this.bytes = new byte[] { mark(52) };
		trace += "body:noarg;";
	}

	public DuplicateWriteByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(51) };
		trace += "body:int:" + marker + ";";
	}
}
