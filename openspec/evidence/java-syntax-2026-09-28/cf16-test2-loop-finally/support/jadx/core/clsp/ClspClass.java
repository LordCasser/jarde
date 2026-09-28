package jadx.core.clsp;

import jadx.core.dex.instructions.args.ArgType;

public final class ClspClass {
	private final String name;
	private final ArgType[] parents;
	public int nameReads;
	public int parentReads;

	public ClspClass(String name, ArgType... parents) {
		this.name = name;
		this.parents = parents;
	}
	public String getName() { nameReads++; return name; }
	public ArgType[] getParents() { parentReads++; return parents; }
}
