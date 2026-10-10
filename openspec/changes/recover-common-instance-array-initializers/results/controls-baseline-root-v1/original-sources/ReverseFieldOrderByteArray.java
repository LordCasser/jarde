public class ReverseFieldOrderByteArray {
	static String trace = "";
	byte[] first;
	byte[] second;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public ReverseFieldOrderByteArray() {
		super();
		this.second = new byte[] { mark(82) };
		this.first = new byte[] { mark(81) };
		trace += "body:noarg;";
	}

	public ReverseFieldOrderByteArray(int marker) {
		super();
		this.second = new byte[] { mark(82) };
		this.first = new byte[] { mark(81) };
		trace += "body:int:" + marker + ";";
	}
}
