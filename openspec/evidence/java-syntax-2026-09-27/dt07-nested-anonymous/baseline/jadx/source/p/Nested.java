package p;

/* JADX INFO: loaded from: input.jar:p/Nested.class */
public final class Nested {
    public static int trace;

    public static Factory create() {
        return new Factory() { // from class: p.Nested.1
            @Override // p.Factory
            public Action make() {
                return new Action() { // from class: p.Nested.1.1
                    @Override // p.Action
                    public void run() {
                        Nested.trace++;
                    }
                };
            }
        };
    }
}
