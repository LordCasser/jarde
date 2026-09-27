package em02;
public class PackageBase {
    protected int stamp;
    void onlyHere() { stamp = 1; }
    public int callBase() { onlyHere(); return stamp; }
}
