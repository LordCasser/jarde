package em05;

/* JADX INFO: loaded from: fixture.jar:em05/Factory.class */
public class Factory {
    public static Child choose(boolean z) {
        return z ? new Child() : new Child("b");
    }

    public static Child chain(String str) {
        return new Child(str);
    }
}
