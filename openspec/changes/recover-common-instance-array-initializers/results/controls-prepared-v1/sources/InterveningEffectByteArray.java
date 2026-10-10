public class InterveningEffectByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public InterveningEffectByteArray() {
		super();
		trace += "gap;";
		this.bytes = new byte[] { mark(61) };
		trace += "body:noarg;";
	}

	public InterveningEffectByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(61) };
		trace += "body:int:" + marker + ";";
	}
}
