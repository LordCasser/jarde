public class FinalLiteralTwoArrays {
	static String trace = "";
	final byte[] first;
	final byte[] second;

	public FinalLiteralTwoArrays() {
		super();
		this.first = new byte[] { 1, 2 };
		this.second = new byte[] { 3, 4 };
		trace += "body:noarg;";
	}

	public FinalLiteralTwoArrays(int marker) {
		super();
		this.first = new byte[] { 1, 2 };
		this.second = new byte[] { 3, 4 };
		trace += "body:int:" + marker + ";";
	}
}
