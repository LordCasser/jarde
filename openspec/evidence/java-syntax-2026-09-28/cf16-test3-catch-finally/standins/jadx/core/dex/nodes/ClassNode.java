package jadx.core.dex.nodes;

public class ClassNode {
    public int loadCount;
    public int unloadCount;

    public void load() {
        loadCount++;
    }

    public void unload() {
        unloadCount++;
    }

    @Override
    public String toString() {
        return "ClassNode";
    }
}
