public class ParameterRhsByteArray {
	static String trace = "";
	byte[] bytes;

	static byte mark(int value) {
		trace += "eval:" + value + ";";
		return (byte) value;
	}

	public ParameterRhsByteArray() {
		super();
		this.bytes = new byte[] { mark(71) };
		trace += "body:noarg;";
	}

	public ParameterRhsByteArray(int marker) {
		super();
		this.bytes = new byte[] { mark(marker) };
		trace += "body:int:" + marker + ";";
	}
}
