public class CommonDirectSuperByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public CommonDirectSuperByteArray() {
		super();
		this.bytes = new byte[] { mark(10), mark(20) };
		trace += "body:noarg;";
	}

	public CommonDirectSuperByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(10), mark(20) };
		trace += "body:int:" + marker + ";";
	}
}
