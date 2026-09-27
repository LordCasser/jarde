package em02;

/* JADX INFO: loaded from: fixture.jar:em02/PackageBase.class */
public class PackageBase {
    protected int stamp;

    void onlyHere() {
        this.stamp = 1;
    }

    public int callBase() {
        onlyHere();
        return this.stamp;
    }
}
