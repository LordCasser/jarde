public class DifferentRhsByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public DifferentRhsByteArray() {
		super();
		this.bytes = new byte[] { mark(31) };
		trace += "body:noarg;";
	}

	public DifferentRhsByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(32) };
		trace += "body:int:" + marker + ";";
	}
}
