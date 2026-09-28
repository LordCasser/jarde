package jadx.core.dex.instructions.args;

public final class ArgType {
	private final String object;
	public int objectReads;
	public static int totalReads;
	public ArgType(String object) { this.object = object; }
	public String getObject() { objectReads++; totalReads++; return object; }
}
